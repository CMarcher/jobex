use std::error::Error;
use std::sync::{Mutex};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use once_cell::unsync::Lazy;
use regex::Regex;
use crate::browser::Browser;

#[async_trait]
pub trait Scraper {
    async fn get_job_count(&self, job_title: &str) -> Result<Option<usize>, Box<dyn Error>>;
}

pub struct IndeedScraper<'a> {
    browser: &'a Browser
}

impl<'a> IndeedScraper<'a> {
    pub fn new(browser: &'a Browser) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl<'a> Scraper for IndeedScraper<'a> {
    async fn get_job_count(&self, job_title: &str) -> Result<Option<usize>, Box<dyn Error>> {
        let page = self.browser.create_page().await?;

        let job_query_string = convert_job_title_to_query_string(job_title, "+");

        page.goto(format!("https://nz.indeed.com/jobs?q={}", job_query_string)).await?;
        page.wait_for_navigation_response().await?;

        let job_count_div = page.find_element("div.jobsearch-JobCountAndSortPane-jobCount").await?;
        let job_count_span = job_count_div.find_element("span").await?;
        let count_text = job_count_span.inner_html().await?;

        let Some(count_text) = count_text else {
            return Ok(None);
        };

        let count = count_text
            .split_whitespace()
            .next()
            .ok_or("No job count text found")?
            .trim()
            .parse::<usize>()?;

        Ok(Some(count))
    }
}

pub struct SeekScraper {
    http_client: Client
}

impl SeekScraper {
    pub fn new(http_client: Client) -> Self {
        Self { http_client }
    }
}

#[async_trait]
impl Scraper for SeekScraper {
    async fn get_job_count(&self, job_title: &str) -> Result<Option<usize>, Box<dyn Error>> {
        let job_query_string = convert_job_title_to_query_string(job_title, "+");
        let request_url = format!("https://jobsearch-api-ts.cloud.seek.com.au/v4/counts?siteKey=NZ-Main&where=All+New+Zealand&keywords={}", job_query_string);
        let response = self.http_client.get(request_url).send().await?;

        let text = response.text().await?;
        let json: Value = serde_json::from_str(&text)?;

        let job_counts = &json["counts"][2]["items"].as_object();
        let job_counts = job_counts.ok_or("Failed to parse JSON returned from Seek job counts API.")?;

        let total = job_counts.values()
            .map(|value| value.as_u64())
            .collect::<Option<Vec<_>>>()
            .ok_or("Failed to parse list of job counts from Seek job count response.")?
            .iter()
            .sum::<u64>();

        Ok(Some(total as usize))
    }
}

pub struct JoraScraper;

#[async_trait]
impl Scraper for JoraScraper {
    async fn get_job_count(&self, job_title: &str) -> Result<Option<usize>, Box<dyn Error>> {
        todo!()
    }
}

pub struct TradeMeScraper<'a> {
    browser: &'a Browser
}

impl<'a> TradeMeScraper<'a> {
    pub fn new(browser: &'a Browser) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl<'a> Scraper for TradeMeScraper<'a> {
    async fn get_job_count(&self, job_title: &str) -> Result<Option<usize>, Box<dyn Error>> {
        let page = self.browser.create_page().await?;
        let job_query_string = convert_job_title_to_query_string(job_title, "%20");

        page.goto(format!("https://www.trademe.co.nz/a/jobs/search?search_string={}", job_query_string)).await?;
        page.wait_for_navigation_response().await?;

        let job_count_h3 = page.find_element("h3.tm-search-header-result-count__heading").await?;
        let count_text = job_count_h3.inner_text().await?;

        static digit_matcher: Mutex<Lazy<Regex>> = Mutex::new(Lazy::new(|| Regex::new(r"\d+").unwrap()));

        let Some(count_text) = count_text else {
            return Ok(None);
        };

        let search_result = digit_matcher.lock()?
            .find(&count_text)
            .ok_or("Could not extract job count from TradeMe search.")?
            .as_str()
            .parse::<usize>()?;
        
        Ok(Some(search_result))
    }
}

fn convert_job_title_to_query_string(job_title: &str, space_replacement: &str) -> String {
    job_title.replace(' ', space_replacement)
}

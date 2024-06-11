use std::error::Error;
use std::sync::Arc;
use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide::cdp::browser_protocol::fetch::{ContinueRequestParams, EventRequestPaused};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let browser_config = BrowserConfig::builder()
        .window_size(1920, 1080)
        .build()?;

    let (mut browser, mut handler) = Browser::launch(browser_config).await?;

    let handle = tokio::task::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });

    let page = Arc::new(browser.new_page("about:blank").await?);
    page.enable_stealth_mode_with_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Safari/605.1.15").await?;
    
    let jobs = [
        "software engineer", 
        "software developer", 
        "civil engineer",
        "web developer",
        "accountant",
        "financial adviser",
        "policy advisor",
        "office administrator",
        "human resources",
        "recruiter"
    ].map(|job| job.replace(' ', "+"));
    
    for job in jobs {
        let job_count = get_job_count(&page, &job).await;
        
        match job_count {
            Ok(Some(count)) => println!("Job count: {}", count),
            _ => println!("Job count: None found")
        }
    }

    browser.close().await?;
    browser.wait().await?;
    handle.await?;

    Ok(())
}

async fn get_job_count(page: &Page, job_title: &str) -> Result<Option<u32>, Box<dyn Error>> {
    page.goto("https://nz.indeed.com/jobs?q={}").await?;
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
        .parse::<u32>()?;
    
    Ok(Some(count))
}

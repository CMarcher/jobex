use std::error::Error;
use std::time::Duration;
use chromiumoxide::cdp::browser_protocol::fetch::{ContinueRequestParams, EventRequestPaused};
use reqwest::Client;
use tokio::join;
use tokio::time::sleep;
use jobex_scraper::browser::Browser;
use jobex_scraper::scrapers::{IndeedScraper, Scraper, SeekScraper, TradeMeScraper};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let browser = Browser::start_context().await?;
    
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
    ];
    
    let indeed_scraper = IndeedScraper::new(&browser);
    let seek_scraper = SeekScraper::new(Client::default());
    let trade_me_scraper = TradeMeScraper::new(&browser);
    
    for job in jobs {
        let (job_count, _) = join!(
            trade_me_scraper.get_job_count(job),
            sleep(Duration::from_millis(500))
        );

        match job_count {
            Ok(Some(count)) => println!("Job count for {}: {}", job, count),
            _ => println!("Job count for {}: None found", job)
        }
    }
    
    Ok(())
}

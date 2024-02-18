use std::error::Error;
use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let browser_config = BrowserConfig::builder().build()?;
    let (mut browser, mut handler) = Browser::launch(browser_config).await?;

    let handle = tokio::task::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });

    let now = std::time::Instant::now();

    let page = browser.new_page("https://www.rust-lang.org").await?;
    let page2 = browser.new_page("https://www.wikipedia.org").await?;
    let user_agent = page.user_agent().await?;
    let content_size = page2.content().await?.len();

    println!("User agent: {}", user_agent);
    println!("Wikipedia content size: {}", content_size);
    println!("Elapsed: {:?}", now.elapsed());

    browser.close().await?;
    browser.wait().await?;

    handle.await?;

    Ok(())
}

use std::error::Error;
use std::sync::Arc;
use chromiumoxide::browser::Browser as ChromiumBrowser;
use chromiumoxide::{BrowserConfig, Page};
use tokio::task::JoinHandle;
use futures::StreamExt;

pub struct Browser {
    browser: ChromiumBrowser,
    handle: JoinHandle<()>
}

impl Browser {
    pub async fn start_context() -> Result<Browser, Box<dyn Error>> {
        let browser_config = BrowserConfig::builder()
            .window_size(1920, 1080)
            .build()?;

        let (inner_browser, mut handler) = chromiumoxide::Browser::launch(browser_config).await?;

        let handle = tokio::task::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });
        
        let browser = Browser { browser: inner_browser, handle };
        
        Ok(browser)
    }
    
    pub async fn create_page(&self) -> Result<Page, Box<dyn Error>> {
        let page = self.browser.new_page("about:blank").await?;
        page.enable_stealth_mode_with_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Safari/605.1.15").await?;
        
        Ok(page)
    }
    
    pub async fn stop_context(mut self) -> Result<(), Box<dyn Error>> {
        self.browser.close().await?;
        self.browser.wait().await?;
        self.handle.await?;
        
        Ok(())
    }
}
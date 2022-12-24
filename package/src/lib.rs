
pub mod service;
pub mod cache;

use std::{env, fs};
use std::thread::sleep;
use std::time::Duration;

use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions, util};
use headless_chrome::browser::default_executable;

pub async fn link_to_text(link: &str) -> anyhow::Result<String> { 

    let path = default_executable().unwrap();

    println!("{:?}",path);

    let mut launch_options = LaunchOptions::default_builder()
        .path(Some(path))
        .sandbox(false)
        .idle_browser_timeout(Duration::from_secs(60))
        .build()
        .unwrap();

    println!("launch_options available");
    let browser = Browser::new(launch_options)?;
    println!("browser available");
    let tab = browser.wait_for_initial_tab()?;
    tab.set_default_timeout(Duration::from_secs(2*60));
    println!("tab available");

    let website = tab
        .navigate_to(link)?.wait_until_navigated()?;

    println!("website available");
    website.wait_for_element_with_custom_timeout("body",Duration::from_secs(60))?;
    println!("body available");

    let mut is_ready = 0usize;
    let mut body_text = "".to_string();
    const TIMEOUT: u64 = 60000; // timeout after 60 seconds
    let mut elapsed_time = 0;

    // if the text did not change for 20s
    while is_ready < 200 || body_text.is_empty() {
        let remote_object = website.wait_for_element("body")?.call_js_fn(r#"
            function is_ready() {
                if (document.readyState === "complete") {
                    // Page is fully loaded
                    return document.body.textContent;
                } else {
                    // Page is not fully loaded
                    return "";
                }
            }
        "#, vec![], false)?;

        match remote_object.value {
            Some(returned_string) => {
                let new_body_text = format!("{}",&returned_string);
                if body_text != new_body_text {
                    is_ready = 0usize;
                    body_text = new_body_text;
                }else{
                    is_ready = is_ready +1usize;
                }
            }
            _ => unreachable!()
        };
        println!("document.readyState === \"complete\": {}",is_ready);

        sleep(Duration::from_millis(100));
        elapsed_time += 100;
        if elapsed_time > TIMEOUT {
            // Timed out, break out of the loop
            break;
        }
    }
    println!("elapsed_time loading: {}",elapsed_time as f64 / 1000f64);

    website.wait_for_element("body")?.call_js_fn(r#"
            function text_only_mode() {
                // Remove all elements that should not be included in the text-only view
                var elements = document.querySelectorAll("button, input, img, svg");
                for (var i = 0; i < elements.length; i++) {
                    elements[i].remove();
                }

                // Disable all stylesheets and remove inline styles
                var stylesheets = document.styleSheets;
                for (var i = 0; i < stylesheets.length; i++) {
                    stylesheets[i].disabled = true;
                }
                document.body.removeAttribute("style");

                var target = document.querySelectorAll('div, p');
                Array.prototype.forEach.call(target, function(element){
                    element.removeAttribute('style');
                });
            }
        "#, vec![], false)?;

    println!("text_only_mode() done");

    let remote_object = website.wait_for_element("body")?.call_js_fn(r#"
            function get_text() {
                let allText = document.body.innerText.trim();
                return allText;
            }
        "#, vec![], false)?;

    match remote_object.value {
        Some(returned_string) => {

            let escaped = returned_string.to_string();
            return Ok(escaped);

        }
        _ => unreachable!()
    };

    /*
    let pdf_options: Option<PrintToPdfOptions> = None; // use chrome's defaults for this example
    let website = website.print_to_pdf(pdf_options)?;
    fs::write("website.pdf", &website)?;
    println!("PDF successfully created from internet web page.");
    */

}
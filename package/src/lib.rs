extern crate core;

pub mod service;
pub mod cache;

use std::{env, fs};
use std::thread::sleep;
use std::time::Duration;

use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions, util};
use headless_chrome::browser::default_executable;
use serde_json::Value;

pub async fn link_to_text(link: &str) -> anyhow::Result<(Vec<String>,Vec<Vec<bool>>)> {


    println!("link: {:?}",link);

    let path = default_executable()?;

    let mut launch_options = LaunchOptions::default_builder()
        .path(Some(path))
        .sandbox(false)
        .idle_browser_timeout(Duration::from_secs(60))
        .build()?;

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
    while is_ready < 100 || body_text.is_empty() {
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

    let remote_object = website.wait_for_element("body")?.call_js_fn(r#"
            function get_text() {
                const buttons = document.querySelectorAll('.show-more-button');

                buttons.forEach((button) => {
                  button.click();
                });

                const inner_text = document.body.innerText;
                const divElements = document.querySelectorAll('div');
                const divArray = Array.from(divElements);

                const leafDivs = divArray.filter((div) => {
                  return !div.querySelector('div') && ((inner_text.includes(div.innerText) && div.innerText.length >=32) || (div.querySelector('p') || div.querySelector('h1')  || div.querySelector('span')));
                });

                const lucky_parents = divArray.filter((div) => {
                  var c = false;
                  for (const d of leafDivs) {
                        if (div.contains(d)) {
                            c = true;
                      break;
                        }
                    }
                  return c;
                });

                // iterate over the list of elements
                lucky_parents.forEach((div) => {
                  // remove the style attribute
                  div.removeAttribute('style');
                  div.removeAttribute('id');

                  // remove all other attributes
                  const attributes = div.attributes;
                  for (let i = 0; i < attributes.length; i++) {
                    div.removeAttribute(attributes[i].name);
                  }
                });

                const toRemove = [...divElements].filter((div) => !lucky_parents.includes(div));

                // remove the elements from the document
                toRemove.forEach((div) => div.remove());

                // set a flag to true to enter the loop
                var found = true;

                // run the loop until no more elements are found
                while (found) {
                  // find the first element that only contains one <div> element
                  var div_found = Array.from(document.querySelectorAll('div')).find((div) => {

                    // check if there is only one child node and it is a <div> element
                    return div.childNodes.length === 1;
                  });

                  // check if an element was found
                  if (div_found) {
                    div_found.replaceWith(div_found.childNodes[0]);
                  } else {
                    // set the flag to false to exit the loop
                    found = false;
                  }
                }
                // get a list of all elements in the document
                const elements = document.querySelectorAll('*');

                // filter the elements to exclude those with no text content
                const noTextElements = Array.from(elements).filter((element) => {
                  return !element.innerText;
                });

                // remove the elements from the document
                noTextElements.forEach((element) => {
                  element.remove();
                });

                const currentdivElements = document.querySelectorAll('div');
                const currentdivArray = Array.from(currentdivElements);
                const currentleafDivs = currentdivArray.filter((div) => {
                  return !div.querySelector('div');
                });

               const queue = [document.querySelector('div')]; // initialize the queue with the root element
               var levels = [];
                while (queue.length > 0) {
                    const div = queue.shift(); // get the next element from the queue
                    console.log(div.innerText); // process the current element

                    var contained_divs = div.querySelectorAll('div');
                    levels.push(currentleafDivs.map(div => {
                        let found = false;
                        contained_divs.forEach(each => {
                            if (each === div) {
                                found = true;
                                return;
                            }
                        });
                        return found;
                    }));

                    // add all children of the current element to the queue
                    for (const child of div.children) {
                        if (child.tagName === 'DIV') {
                            queue.push(child);
                        }
                    }
                }
                console.log(levels);

                return JSON.stringify({"text_nodes": currentleafDivs.map(div => div.innerText), "hierarchical_segmentation": levels.filter(a => a.includes(true))});
                //return document.body.innerHTML;
            }
        "#, vec![], false)?;

    let err_msg = anyhow::anyhow!(format!("Error: Unreachable: Unable to load webpage: {}", link));

    match remote_object.value {
        Some(returned_string) => {
            let val: Value = serde_json::from_str(returned_string.as_str().ok_or(Err(err_msg.clone()))?)?;
            println!("{:?}", val);

            let text_nodes = val
                .get("text_nodes").ok_or(Err(err_msg.clone()))?
                .as_array().ok_or(Err(err_msg.clone()))?
                .into_iter()
                .map(|y| y.as_str().ok_or(Err(err_msg.clone()))?.to_string())
                .collect::<Vec<String>>();

            let hierarchical_segmentation = val
                .get("hierarchical_segmentation").ok_or(Err(err_msg.clone()))?
                .as_array().ok_or(Err(err_msg.clone()))?
                .into_iter()
                .map(|y| y.as_array().ok_or(Err(err_msg.clone()))?
                    .into_iter().map(|x| x.as_bool().ok_or(Err(err_msg.clone()))?)
                    .collect::<Vec<bool>>())
                .collect::<Vec<Vec<bool>>>();

            Ok((text_nodes, hierarchical_segmentation))
        }
        None => Err(err_msg)
    }

    /*
    let pdf_options: Option<PrintToPdfOptions> = None; // use chrome's defaults for this example
    let website = website.print_to_pdf(pdf_options)?;
    fs::write("website.pdf", &website)?;
    println!("PDF successfully created from internet web page.");
    */

}
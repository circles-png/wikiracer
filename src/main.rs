use inquire::Text;
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::{HashMap, VecDeque};

fn get_articles() -> (String, String) {
    println!("prompting for articles...");
    let prompt = Text::new("")
        .with_initial_value("https://en.wikipedia.org/wiki/")
        .with_placeholder("https://en.wikipedia.org/wiki/");
    let start = {
        let mut prompt = prompt.clone();
        prompt.message = "which article should we start at?";
        prompt.prompt()
    }
    .expect("start article should be able to be read");
    let end = {
        let mut prompt = prompt.clone();
        prompt.message = "which article should we end at?";
        prompt.prompt()
    }
    .expect("end article should be able to be read");
    println!("done!");
    (start, end)
}

fn get_links(page: &String) -> Vec<String> {
    println!("getting links from {}...", page);
    println!("parsing page...");
    let html = Html::parse_document(
        get(page)
            .expect("page should be able to be read")
            .text()
            .expect("page text should be able to be read")
            .as_str(),
    );
    println!("getting base url...");
    let base_url = page
        .split("/wiki/")
        .next()
        .expect("page should have a base url");
    println!("finding links...");
    let links = html
        .select(&Selector::parse("p a[href]").expect("selector parsing should work"))
        .filter_map(|link| {
            if link
                .value()
                .attr("href")
                .expect("link should have an href")
                .starts_with("/wiki/")
            {
                Some(format!(
                    "{}{}",
                    base_url,
                    link.value().attr("href").unwrap()
                ))
            } else {
                None
            }
        })
        .collect();
    println!("done!");
    links
}

fn check_articles(start: &String, end: &String) {
    println!("checking articles...");
    println!("getting languages...");
    let mut languages = Vec::new();
    for page in [start, end] {
        let language = Regex::new(r"https://([a-z]{2,3})\.wikipedia\.org/wiki/")
            .expect("language regex should be able to be compiled")
            .captures(page)
            .expect("language regex should find a match")
            .get(1)
            .expect("language regex should have a second capture group")
            .as_str();
        if !languages.contains(&language) {
            languages.push(language);
        }
        get(page).expect("page should be able to be read");
    }
    println!("checking languages...");
    if languages.len() > 1 {
        panic!("start and end articles should be in the same language")
    }
    println!("checking start article...");
    if get_links(start).is_empty() {
        panic!("start article should have links")
    }

    println!("checking end article...");
    if Html::parse_document(
        get(end)
            .unwrap()
            .text()
            .expect("end article text should be able to be read")
            .as_str(),
    )
    .select(
        &Selector::parse("table.plainlinks.metadata.ambox.ambox-style.ambox-Orphan")
            .expect("selector parsing should work"),
    )
    .count()
        > 0
    {
        panic!("end article should not be an orphan")
    }
    println!("done!");
}

fn find_shortest_path(start: &str, end: &String) -> Option<Vec<String>> {
    println!("finding shortest path...");
    let mut path = HashMap::new();
    path.insert(start.to_owned(), vec![start.to_owned()]);
    let mut queue = VecDeque::new();
    queue.push_back(start.to_owned());

    while !queue.is_empty() {
        let page = queue.pop_front().expect("queue should not be empty");
        let links = get_links(&page);

        for link in links {
            let link = link.clone();
            if link == *end {
                return {
                    let mut links: Vec<String> = path
                        .get(&page)
                        .expect("page should be in path")
                        .to_vec();
                    links.push(link);
                    Some(links)
                };
            }

            if (!path.contains_key(&link)) && (link != *page) {
                path.insert(link.clone(), {
                    let mut links: Vec<String> = path
                        .get(&page)
                        .expect("page should be in path")
                        .to_vec();
                    links.push(link.clone());
                    links
                });

                queue.push_back(link)
            }
        }
    }

    None
}

fn redirected(page: &String) -> String {
    println!("redirecting {}...", page);
    let html = Html::parse_document(
        get(page)
            .unwrap()
            .text()
            .expect("page article text should be able to be read")
            .as_str(),
    );
    let title = html
        .select(&Selector::parse("h1").expect("title parsing should work"))
        .next()
        .expect("title should exist")
        .text()
        .next()
        .expect("title text should exist");
    let title = title.replace(' ', "_");
    let base_url = page
        .split("/wiki/")
        .next()
        .expect("page should have a base url")
        .to_owned()
        + "/wiki/";
    println!("done!");
    format!("{}{}", base_url, title)
}

fn main() {
    println!("hi! this is wikiracer-rs");
    let (start, end) = get_articles();
    check_articles(&start, &end);
    println!(
        "shortest path: {}",
        find_shortest_path(&start, &redirected(&end))
            .unwrap_or(vec!["no path found".to_string()])
            .join("\n    to ")
    );
}

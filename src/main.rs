use clap::Parser;
use select::{document::Document, predicate::Name};
use surf;

#[derive(Debug, Parser)]
struct Arguments {
    #[clap(short='t', long="site", env="SITE_URI")]
    uri: String,

    #[clap(short='x', long="exclude")]
    exclude: Option<Vec<String>>,
}

async fn body(uri: String) -> Result<String, surf::Error> {
    let client = surf::Client::new().with(surf::middleware::Redirect::new(5));
    let req = client.get(uri).recv_string();
    req.await
}

fn extract_internal_urls(body: &str) -> Vec<String> {
    let doc = Document::from(body);
    let urls = doc.find(Name("a"))
        .filter_map(|x| x.attr("href"))
        .filter(|x| x.starts_with("/") || x.starts_with("https://ntalbs.github.io"))
        .map(|x| x.to_string())
        .collect();
    urls
}

fn main() {
    let args = Arguments::parse();
    let uri = args.uri;
    println!("Target site: {}", uri);
    if let Some(exclude) = args.exclude {
        println!("excludes: {:?}", exclude);
    }


    let response = async_std::task::block_on(body(uri));
    let body = match response {
        Ok(body) => body,
        Err(_) => panic!("Error"),
    };

    let anchors = extract_internal_urls(body.as_str());

    anchors.iter().for_each(|x| println!("{}", x));
}

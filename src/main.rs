use muy_zipido::MuyZipido;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://opendata.manage-roadworks.service.gov.uk/permit/2020/07.zip";

    println!("Fetching and processing ZIP from: {}", url);

    let mut extractor = MuyZipido::new(url, 8192)?;
    let entries = extractor.process()?;

    println!("\n=== Summary ===");
    println!("Total entries: {}", entries.len());

    Ok(())
}

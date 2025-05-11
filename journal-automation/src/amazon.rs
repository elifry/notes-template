use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

use crate::utils::get_journal_path_for_date;

#[derive(Debug, Deserialize)]
struct DigitalItem {
    #[serde(rename = "ProductName")]
    title: String,
    #[serde(rename = "OrderDate")]
    order_date: String,
    #[serde(rename = "OurPrice")]
    price: String,
}

#[derive(Debug, Deserialize)]
struct RetailItem {
    #[serde(rename = "Order Date")]
    order_date: String,
    #[serde(rename = "Total Owed")]
    total_owed: String,
    #[serde(rename = "Product Name")]
    product_name: String,
    #[serde(rename = "Website")]
    website: String,
}

#[derive(Debug, Deserialize)]
struct ReturnItem {
    #[serde(rename = "Return Requested Date")]
    return_date: String,
    #[serde(rename = "Product Name")]
    product_name: String,
    #[serde(rename = "Return Reason Code")]
    return_reason: String,
}

#[derive(Debug, Deserialize)]
struct BorrowedItem {
    #[serde(rename = "ProductName")]
    title: String,
    #[serde(rename = "Author")]
    author: String,
    #[serde(rename = "LoanCreationDate")]
    borrow_date: String,
}

pub enum AmazonDataPath {
    DigitalItems,
    RetailOrders,
    Returns,
    DigitalBorrows,
}

#[derive(Debug, Clone)]
pub struct ProcessedOrder {
    pub name: String,
    pub price: f64,
    pub purchase_type: PurchaseType,
}

#[derive(Debug, Clone)]
pub enum PurchaseType {
    Digital,
    Audible,
    AmazonRetail,
    WholeFoods,
    AmazonRental,
}

impl fmt::Display for PurchaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurchaseType::Digital => write!(f, "Digital Orders"),
            PurchaseType::Audible => write!(f, "Audible"),
            PurchaseType::AmazonRetail => write!(f, "Amazon.com"),
            PurchaseType::WholeFoods => write!(f, "Whole Foods"),
            PurchaseType::AmazonRental => write!(f, "Amazon Rentals"),
        }
    }
}

#[derive(Debug)]
pub enum AmazonActivity {
    Purchase(ProcessedOrder),
    Return {
        product_name: String,
        reason: String,
    },
    Borrow {
        title: String,
        author: String,
    },
}

impl AmazonDataPath {
    pub fn path(&self, data_dir: &str) -> String {
        match self {
            AmazonDataPath::DigitalItems => {
                format!("{}/Digital-Ordering.1/Digital Items.csv", data_dir)
            }
            AmazonDataPath::RetailOrders => {
                format!(
                    "{}/Retail.OrderHistory.1/Retail.OrderHistory.1.csv",
                    data_dir
                )
            }
            AmazonDataPath::Returns => {
                format!(
                    "{}/Retail.Orders.ManageYourReturns.1/Retail.Orders.ManageYourReturns.1.csv",
                    data_dir
                )
            }
            AmazonDataPath::DigitalBorrows => {
                format!("{}/Digital.Borrows.1/Digital.Borrows.1.csv", data_dir)
            }
        }
    }
}

fn determine_purchase_type(product_name: &str, is_digital: bool, website: &str) -> PurchaseType {
    if is_digital {
        if product_name.to_lowercase().contains("audible")
            || product_name.to_lowercase().contains("audiobook")
        {
            PurchaseType::Audible
        } else {
            PurchaseType::Digital
        }
    } else {
        if website == "panda01" || product_name.to_lowercase().contains("whole foods") {
            PurchaseType::WholeFoods
        } else if product_name.to_lowercase().contains("rental")
            || product_name.to_lowercase().contains("rent")
        {
            PurchaseType::AmazonRental
        } else {
            PurchaseType::AmazonRetail
        }
    }
}

fn append_or_update_journal(
    journal_path: &str,
    date: NaiveDate,
    content: &str,
    verbose: bool,
) -> Result<bool> {
    let file_exists = std::path::Path::new(journal_path).exists();
    let mut file_content = if file_exists {
        std::fs::read_to_string(journal_path)?
    } else {
        String::new()
    };

    // If file is empty, add the date header
    if file_content.trim().is_empty() {
        file_content = format!("# {}\n", date.format("%A, %B %d, %Y"));
    }

    // Check if Amazon section exists
    if let Some(amazon_start) = file_content.find("\n## Amazon Activity (AUTOMATED)") {
        // Find the end of the Amazon section (next ## or end of file)
        let amazon_end = file_content[amazon_start + 1..]
            .find("\n## ")
            .map(|pos| amazon_start + 1 + pos)
            .unwrap_or(file_content.len());

        // Compare existing content with new content
        let existing_section = &file_content[amazon_start..amazon_end];
        if existing_section.trim() == content.trim() {
            if verbose {
                println!("Amazon section already up to date, no changes needed.");
            }
            return Ok(false);
        }

        // Replace the existing section
        file_content = format!(
            "{}{}{}",
            &file_content[..amazon_start],
            content,
            &file_content[amazon_end..]
        );
    } else {
        // No existing section, append the new content
        if !file_content.ends_with('\n') {
            file_content.push('\n');
        }
        file_content.push_str(content);
    }

    // Write the updated content back to file
    std::fs::write(journal_path, file_content)?;
    Ok(true)
}

pub fn format_activities(activities: &[AmazonActivity]) -> String {
    if activities.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n## Amazon Activity (AUTOMATED)\n\n");

    // Group activities by type
    let mut purchases: Vec<&ProcessedOrder> = Vec::new();
    let mut returns: Vec<(&String, &String)> = Vec::new();
    let mut borrows: Vec<(&String, &String)> = Vec::new();

    // Collect activities into their respective groups
    for activity in activities {
        match activity {
            AmazonActivity::Purchase(order) => purchases.push(order),
            AmazonActivity::Return {
                product_name,
                reason,
            } => returns.push((product_name, reason)),
            AmazonActivity::Borrow { title, author } => borrows.push((title, author)),
        }
    }

    // Format purchases by type
    let mut purchase_groups: Vec<(&PurchaseType, Vec<&ProcessedOrder>)> = Vec::new();
    for order in purchases {
        let group = purchase_groups.iter_mut().find(|(t, _)| {
            std::mem::discriminant(*t) == std::mem::discriminant(&order.purchase_type)
        });

        match group {
            Some((_, orders)) => orders.push(order),
            None => purchase_groups.push((&order.purchase_type, vec![order])),
        }
    }

    // Sort purchase groups by their display name
    purchase_groups.sort_by(|(a, _), (b, _)| a.to_string().cmp(&b.to_string()));

    // Output purchases
    for (purchase_type, mut orders) in purchase_groups {
        output.push_str(&format!("- {}\n", purchase_type));
        // Sort orders by name
        orders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        for order in orders {
            if order.price == 0.0 {
                output.push_str(&format!("  - {}\n", order.name));
            } else {
                output.push_str(&format!("  - {} (${:.2})\n", order.name, order.price));
            }
        }
    }

    // Output returns (sorted by product name)
    if !returns.is_empty() {
        output.push_str("- Returned\n");
        let mut sorted_returns = returns;
        sorted_returns
            .sort_by(|(a_name, _), (b_name, _)| a_name.to_lowercase().cmp(&b_name.to_lowercase()));
        for (name, reason) in sorted_returns {
            output.push_str(&format!("  - {} (Reason: {})\n", name, reason));
        }
    }

    // Output borrows (sorted by title)
    if !borrows.is_empty() {
        output.push_str("- Borrowed\n");
        let mut sorted_borrows = borrows;
        sorted_borrows.sort_by(|(a_title, _), (b_title, _)| {
            a_title.to_lowercase().cmp(&b_title.to_lowercase())
        });
        for (title, author) in sorted_borrows {
            output.push_str(&format!("  - {} by {}\n", title, author));
        }
    }

    output
}

pub fn analyze_amazon_data(data_dir: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("\nAmazon Data Analysis");
        println!("===================================");
    }

    let mut activities_by_date: HashMap<NaiveDate, Vec<AmazonActivity>> = HashMap::new();
    let mut activity_counts = (0, 0, 0, 0); // (digital, retail, returns, borrows)
    let mut files_updated = 0;
    let mut files_unchanged = 0;

    // Process Digital Items
    let digital_items_path = AmazonDataPath::DigitalItems.path(data_dir);
    if verbose {
        println!("\nLooking for digital items at: {}", digital_items_path);
    }

    if let Ok(mut rdr) = Reader::from_path(&digital_items_path) {
        if verbose {
            println!("Processing digital items...");
        }
        for result in rdr.deserialize::<DigitalItem>() {
            if let Ok(record) = result {
                // Parse the date format: "2024-09-06T02:19:00Z"
                if let Ok(date) =
                    NaiveDateTime::parse_from_str(&record.order_date, "%Y-%m-%dT%H:%M:%SZ")
                        .map(|dt| dt.date())
                {
                    // Handle price as a string that might be "Not Applicable" or empty
                    let price = if record.price == "Not Applicable" || record.price.is_empty() {
                        0.0
                    } else {
                        record.price.parse::<f64>().unwrap_or(0.0)
                    };

                    let order = ProcessedOrder {
                        name: record.title.clone(),
                        price,
                        purchase_type: determine_purchase_type(&record.title, true, ""),
                    };

                    activities_by_date
                        .entry(date)
                        .or_insert_with(Vec::new)
                        .push(AmazonActivity::Purchase(order));
                    activity_counts.0 += 1;
                }
            }
        }
    } else if verbose {
        println!("Could not open digital items file: {}", digital_items_path);
    }

    // Process Retail Orders
    let retail_items_path = AmazonDataPath::RetailOrders.path(data_dir);
    if let Ok(mut rdr) = Reader::from_path(&retail_items_path) {
        if verbose {
            println!("\nProcessing retail orders...");
        }
        for result in rdr.deserialize::<RetailItem>() {
            if let Ok(record) = result {
                if let Ok(date) = NaiveDate::parse_from_str(&record.order_date[..10], "%Y-%m-%d") {
                    let price = record
                        .total_owed
                        .trim_start_matches('$')
                        .parse::<f64>()
                        .unwrap_or(0.0);

                    let order = ProcessedOrder {
                        name: record.product_name.clone(),
                        price,
                        purchase_type: determine_purchase_type(
                            &record.product_name,
                            false,
                            &record.website,
                        ),
                    };

                    activities_by_date
                        .entry(date)
                        .or_insert_with(Vec::new)
                        .push(AmazonActivity::Purchase(order));
                    activity_counts.1 += 1;
                }
            }
        }
    }

    // Process Returns
    let returns_path = AmazonDataPath::Returns.path(data_dir);

    if let Ok(mut rdr) = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&returns_path)
    {
        if verbose {
            println!("\nProcessing returns...");
        }
        for result in rdr.deserialize() {
            let record: ReturnItem = result?;
            if let Ok(date) =
                NaiveDateTime::parse_from_str(&record.return_date, "%Y-%m-%dT%H:%M:%SZ")
                    .map(|dt| dt.date())
            {
                // Map return reason codes to human-readable reasons
                let reason = match record.return_reason.as_str() {
                    "CR-DEFECTIVE" => "Item was defective",
                    "AMZ-PG-MISORDERED" => "Wrong item ordered",
                    _ => &record.return_reason,
                };

                activities_by_date
                    .entry(date)
                    .or_insert_with(Vec::new)
                    .push(AmazonActivity::Return {
                        product_name: record.product_name,
                        reason: reason.to_string(),
                    });
                activity_counts.2 += 1;
            }
        }
    }

    // Process Digital Borrows
    let borrows_path = AmazonDataPath::DigitalBorrows.path(data_dir);
    if let Ok(mut rdr) = Reader::from_path(&borrows_path) {
        if verbose {
            println!("\nProcessing digital borrows...");
        }
        for result in rdr.deserialize::<BorrowedItem>() {
            if let Ok(record) = result {
                if let Ok(date) =
                    NaiveDateTime::parse_from_str(&record.borrow_date, "%Y-%m-%dT%H:%M:%SZ")
                        .map(|dt| dt.date())
                {
                    activities_by_date
                        .entry(date)
                        .or_insert_with(Vec::new)
                        .push(AmazonActivity::Borrow {
                            title: record.title,
                            author: record.author,
                        });
                    activity_counts.3 += 1;
                }
            }
        }
    }

    if verbose {
        println!("\nActivity Summary:");
        println!("- Digital Orders: {}", activity_counts.0);
        println!("- Retail Orders: {}", activity_counts.1);
        println!("- Returns: {}", activity_counts.2);
        println!("- Borrows: {}", activity_counts.3);

        println!("\nAnalysis complete. Making changes to journal files...");
        println!("===================================");
    }

    // Sort dates for consistent output
    let mut dates: Vec<_> = activities_by_date.keys().collect();
    dates.sort();

    // Make all changes
    for &date in &dates {
        let activities = &activities_by_date[date];
        if let Ok(journal_path) = get_journal_path_for_date(*date) {
            if verbose {
                println!(
                    "\nProcessing: {} ({} activities)",
                    journal_path,
                    activities.len()
                );
            }

            let content = format_activities(activities);
            match append_or_update_journal(&journal_path, *date, &content, verbose) {
                Ok(true) => files_updated += 1,
                Ok(false) => files_unchanged += 1,
                Err(e) => println!("Error updating {}: {}", journal_path, e),
            }

            if verbose {
                println!("Activities for {}:", date.format("%Y-%m-%d"));
                for activity in activities {
                    match activity {
                        AmazonActivity::Purchase(order) => println!("  Purchase: {}", order.name),
                        AmazonActivity::Return {
                            product_name,
                            reason,
                        } => {
                            println!("  Return: {} (Reason: {})", product_name, reason);
                        }
                        AmazonActivity::Borrow { title, .. } => println!("  Borrow: {}", title),
                    }
                }
            }
        }
    }

    let total_activities =
        activity_counts.0 + activity_counts.1 + activity_counts.2 + activity_counts.3;
    let unique_days = activities_by_date.len();
    println!("\nProcessed {} activities ({} digital orders, {} retail orders, {} returns, {} borrows, {} unique days with activity)", 
        total_activities, activity_counts.0, activity_counts.1, activity_counts.2, activity_counts.3, unique_days);
    println!(
        "Updated {} journal files ({} unchanged)",
        files_updated, files_unchanged
    );

    Ok(())
}

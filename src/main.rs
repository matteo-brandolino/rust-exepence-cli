use core::fmt;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{self, BufRead as _, Write};
use std::path::Path;
use std::str::FromStr;

fn main() {
    show_welcome_message();
    let expense_file_path: Option<String> = ask_or_get_current_month();
    ask_multiple_choice_question(expense_file_path);
}

fn ask_or_get_current_month() -> Option<String> {
    let expense_file_path: Option<String> = match get_month_file_name() {
        Some(month_file_name) => {
            println!("Month file name: {}", month_file_name);
            return Some(format!("./in_progress/{}.csv", month_file_name));
        }
        None => None,
    };

    if expense_file_path.is_none() {
        let expense_file: String = parse_input(Prompt::Str("Enter the file name"));
        let path = format!("./in_progress/{}.csv", expense_file);
        let _ = save_to_file(&path);
        return Some(path);
    } else {
        panic!("Cannot get or ask the current month file name")
    }
}

fn get_month_file_name() -> Option<String> {
    if let Ok(entries) = fs::read_dir("./in_progress") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    if let Some(name) = file_name.to_str() {
                        if name.ends_with(".csv") {
                            if let Some(file_stem) = path.file_stem() {
                                if let Some(file_stem_str) = file_stem.to_str() {
                                    return Some(String::from(file_stem_str));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn ask_multiple_choice_question(mut expense_file_path: Option<String>) {
    loop {
        main_menu();
        show_right_arrow();

        let mut choice: String = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");
        let choice: &str = choice.trim();

        match choice {
            "1" => {
                if let Some(expense) = add_record() {
                    if let Some(file_path) = &expense_file_path {
                        save_record(&expense, file_path);
                        continue;
                    } else {
                        println!("Expense file path is missing or invalid.");
                        continue;
                    }
                } else {
                    println!("Failed to get expense details.");
                    continue;
                }
            }
            "2" => match create_and_move_file(&expense_file_path) {
                Ok(_) => {
                    println!("Month closed successfully!");
                    expense_file_path = ask_or_get_current_month();
                }
                Err(err) => eprintln!("Error creating or moving the file: {}", err),
            },
            "3" => {
                // stats
                match calculate_sum() {
                    Ok(result) => {
                        for (key, value) in result.iter() {
                            if key.to_string() == "Month" {
                                println!("");
                            }
                            println!("{}: {}", key, value);
                        }
                    }
                    Err(err) => eprintln!("Errore: {}", err),
                }
                continue;
            }
            "4" => {
                //delete category
            }
            "5" => {
                println!("EXITING THE EXPENSE TRACKER! BYE BYE 🖐️");
                break;
            }
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn add_record() -> Option<Expense> {
    println!("\n✍️ ENTER THE EXPENSE/INCOME");

    let expense_name: String = parse_input(Prompt::Str("  Expense/Income name: "));
    let expense_amount: f64 = parse_input(Prompt::Str("  Euro amount: "));

    let categories_result = get_categories();

    let expense_category = match categories_result {
        Ok(categories) => categories,
        Err(_) => Vec::new(), // Se c'è un errore, assegna un vettore vuoto
    };
    loop {
        println!("\n🔢 CHOOSE A CATEGORY");
        for (i, category) in expense_category.iter().enumerate() {
            println!("  {}: {}", i + 1, category);
        }
        let mut category_text = String::new();
        let mut new_category_text = String::new();
        new_category_text.push_str("type a new category");

        if expense_category.len() > 0 {
            let value_range: String = format!("[1 - {}]", expense_category.len());
            category_text = format!("\nEnter a Category {} or ", value_range)
        } else {
            let words: Vec<&str> = new_category_text.split_whitespace().collect();

            let first_letter_in_caps: String = words
                .into_iter()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first_word) => {
                            first_word.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");

            new_category_text = first_letter_in_caps;
        }

        let prompt: Prompt = Prompt::String(format!("{}{} ", category_text, new_category_text));
        let selected_category: CategoryResultType = parse_input(prompt);
        let category: String;

        match selected_category {
            CategoryResultType::StringResult(ref selected_category) => {
                save_new_category(selected_category);
                category = selected_category.clone()
            }
            CategoryResultType::NumberResult(num) if num > 0 && num <= expense_category.len() => {
                category = expense_category[num - 1].clone();
            }
            _ => {
                println!("\nInvalid Entry, Please try again");
                continue;
            }
        }

        println!("\n📝 ENTERED RECORD DETAILS");
        println!("  Record Name: {}", expense_name);
        println!("  Record Amount: {}", expense_amount);
        println!("  Recod Category: {}", category);

        print!("\nENTER THIS RESPONSE (yes/no): ");
        io::stdout().flush().unwrap();

        let mut confirm = String::new();
        io::stdin()
            .read_line(&mut confirm)
            .expect("Failed to read line");
        let confirm: String = confirm.trim().to_lowercase();

        if confirm == "yes" || confirm == "y" {
            let new_expense: Expense = Expense {
                name: expense_name.clone(),
                category,
                amount: expense_amount,
            };
            return Some(new_expense);
        } else {
            return None;
        }
    }
}

fn get_categories() -> io::Result<Vec<String>> {
    let file_path = "./categories.txt";

    // Apri il file in modalità di lettura con un gestore di errori
    let file = File::open(file_path)?;

    // Utilizza un buffer per leggere il file riga per riga
    let reader = io::BufReader::new(file);

    // Dichiara un vettore per contenere le stringhe lette dal file
    let mut lines: Vec<String> = Vec::new();

    // Itera su ogni riga del file e aggiungi la stringa al vettore
    for line in reader.lines() {
        lines.push(line?);
    }

    // Restituisci il vettore contenente le stringhe lette dal file
    Ok(lines)
}
fn save_new_category(selected_category: &String) {
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("categories.txt")
    {
        writeln!(&mut file, "{}", selected_category).expect("Failed to write to file");
    } else {
        eprintln!("Failed to open file for writing");
    }
}
fn save_record(expense: &Expense, expense_file_path: &str) {
    println!(
        "\n📁 SAVING USER EXPENSE : {:?} on {:?} file",
        expense, expense_file_path
    );

    // if folder doesn't esists, create one
    if !Path::new("in_progress").exists() {
        match std::fs::create_dir("in_progress") {
            Ok(_) => println!("\nNew file created"),
            Err(err) => panic!("Fail to create folder: {}", err),
        }
    }

    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(expense_file_path)
    {
        if let Err(err) = writeln!(
            &mut file,
            "{},{},{}",
            expense.name,
            expense.category.to_string(),
            expense.amount
        ) {
            eprintln!("Failed to write to file: {}", err);
        }
    } else {
        eprintln!("Failed to open file for writing");
    }
}
fn save_to_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    File::create(file_path)?;

    println!("File '{}' created successfully ✅", file_path);

    Ok(())
}
fn calculate_sum() -> Result<Vec<(&'static str, ResultType)>, Box<dyn Error>> {
    let months_input: String = parse_input(Prompt::Str("Month Stats"));
    let months: Vec<String> = months_input
        .split(',')
        .map(|s| capitalize_first_char(s.trim()))
        .collect();
    let mut result: Vec<(&str, ResultType)> = Vec::new();

    for month in months {
        let month_file_path = format!("./done/{}.csv", month);
        if let Ok(metadata) = fs::metadata(month_file_path.clone()) {
            if metadata.is_file() {
                let file: File = File::open(month_file_path)?;
                let mut rdr = ReaderBuilder::new()
                    .delimiter(b',')
                    .has_headers(false)
                    .from_reader(file);

                let mut income: f64 = 0.0;
                let mut expense: f64 = 0.0;

                for result in rdr.records() {
                    let record: csv::StringRecord = result?;
                    if let Some(value) = record.get(2) {
                        if let Ok(number) = value.parse::<f64>() {
                            if number > 0.0 {
                                income += number;
                            }
                            if number < 0.0 {
                                expense += number;
                            }
                        }
                    }
                }

                let mut sum: f64 = income + expense;
                sum = (sum * 100.0).round() / 100.0;

                result.push(("Month", ResultType::Text(month.to_string())));
                result.push(("Income", ResultType::Number(income)));
                result.push(("Expense", ResultType::Number(expense)));
                result.push(("Sum", ResultType::Number(sum)));

                let percentage_sum: f64 = ((sum / income) * 100.0).round();
                let formatted_percentage_sum = format!("{}%", percentage_sum);

                result.push(("Percentage", ResultType::Text(formatted_percentage_sum)));
            } else {
                println!(
                    "The path {} corresponds to something else, not to a file",
                    month_file_path
                );
                println!();
            }
        } else {
            println!(
                "The file {} does not exist or an error occurred during the check",
                month
            );
            println!();
        }
    }

    Ok(result)
}

// logging functions
fn main_menu() {
    println!("\n⚖️ CHOOSE AMONG THE FOLLOWING OPTIONS");
    println!("  1. 💵 Enter an Expense/Income");
    println!("  2. 📄 Close current month file");
    println!("  3. 📊 Show Stats)");
    println!("  4. 🗑️ Delete a category (TO DO)");
    println!("  5. ❌ Exit");
}

fn show_right_arrow() {
    print!("-> ");
    io::stdout().flush().unwrap();
}

fn show_welcome_message() {
    println!(
        r#"
        _____  ___ __   ___ _ __  ___  ___  | |_ _ __ __ _  ___| | _____ _ __ 
       / _ \ \/ / '_ \ / _ \ '_ \/ __|/ _ \ | __| '__/ _` |/ __| |/ / _ \ '__|
      |  __/>  <| |_) |  __/ | | \__ \  __/ | |_| | | (_| | (__|   <  __/ |    
       \___/_/\_\ .__/ \___|_| |_|___/\___|  \__|_|  \__,_|\___|_|\_\___|_|    
                |_|                                                                      
    "#
    );
}

// Helpers
#[derive(Debug)]
enum CategoryResultType {
    StringResult(String),
    NumberResult(usize),
}
impl FromStr for CategoryResultType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(index) = s.trim().parse::<usize>() {
            Ok(CategoryResultType::NumberResult(index))
        } else {
            Ok(CategoryResultType::StringResult(s.to_string()))
        }
    }
}
enum Prompt {
    Str(&'static str),
    String(String),
}
#[derive(Debug)]
enum ResultType {
    Number(f64),
    Text(String),
}

impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultType::Number(num) => write!(f, "{}", num),
            ResultType::Text(text) => write!(f, "{}", text),
        }
    }
}
fn parse_input<T: std::str::FromStr>(prompt: Prompt) -> T {
    let prompt_str = match prompt {
        Prompt::Str(s) => s.to_string(),
        Prompt::String(s) => s,
    };
    loop {
        print!("{}: ", &prompt_str);
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed_input = input.trim().to_string();
        match trimmed_input.parse() {
            Ok(value) => return value,
            Err(_) => {
                println!("Invalid input, please try again.");
                continue;
            }
        }
    }
}

fn create_and_move_file(expense_file_path: &Option<String>) -> Result<(), std::io::Error> {
    if let Some(file_path) = expense_file_path {
        create_done_folder()?;
        let file_to_move = Path::new(&file_path);
        move_file(&file_to_move, "done")
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Expense file path is missing or invalid.",
        ))
    }
}

fn create_done_folder() -> Result<(), std::io::Error> {
    let done_folder = Path::new("done");

    if !done_folder.exists() {
        fs::create_dir(done_folder)?;
    }

    Ok(())
}

fn move_file(file_to_move: &Path, destination: &str) -> Result<(), std::io::Error> {
    let destination_folder = Path::new(destination);

    if !destination_folder.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("The '{}' folder does not exist", destination),
        ));
    }

    let new_path = destination_folder.join(file_to_move.file_name().unwrap());

    fs::rename(file_to_move, &new_path)?;

    Ok(())
}
fn capitalize_first_char(s: &str) -> String {
    let mut chars = s.chars();

    if let Some(first_char) = chars.next() {
        let uppercased_first_char = first_char.to_uppercase();

        // Collect the remaining characters and combine them into a new string
        let rest_of_string: String = chars.collect();

        // Combine the uppercase character with the rest of the string
        return uppercased_first_char.to_string() + &rest_of_string;
    }

    // If the string is empty, return the original string
    s.to_string()
}
#[derive(Debug)]
struct Expense {
    name: String,
    category: String,
    amount: f64,
}

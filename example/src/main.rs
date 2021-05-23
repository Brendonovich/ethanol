use ethanol::*;

#[derive(Model)]
pub struct Account {
    pub id: String,
    pub name: String
}

fn main() {
    println!("{:?}", vec![
        Account::id().equals("Test".to_string()),
        Account::name().contains("Test".to_string())
    ]);
}

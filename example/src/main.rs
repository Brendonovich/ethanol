use ethanol::*;

#[derive(Model)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub money: i32,
    pub verified: bool
}

#[derive(Model)]
pub struct Tenant {
    pub id: String,
    pub email: String,
    pub phone: String,
    pub owner: Account
}

fn main() -> Result<(), ()>{
    let client = Client::new();

    client.tenant().find_many(vec![
        Tenant::id().equals("Test".to_string()),
        Tenant::owner().some(vec![
            Account::id().equals("Test".to_string())
        ])
    ]);

    Ok(())
}

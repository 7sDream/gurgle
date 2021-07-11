use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "gurgle.pest"]
pub struct GurgleCommandParser {}

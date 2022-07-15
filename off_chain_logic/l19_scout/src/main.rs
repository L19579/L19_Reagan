use L19_Reagan as L19;

fn main() {
    let mut scout = L19::Scout::new().unwrap();
    loop{
        match scout.launch_searchers(){
            Err(_) => {
                L19::show_statement(L19::StatementType::BoldError
                    , "Unexpected crash. Program restarting.");
                continue;
            }
            _ => break,
        };
    }
}

struct Command {
    a: &'static str
}

impl Command {
    fn fake(&self) -> u32 {
        return 3;
    }
}

//struct ListCommand {
//    act: String
//}

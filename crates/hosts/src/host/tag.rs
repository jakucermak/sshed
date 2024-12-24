pub mod tag {

    #[derive(Debug)]
    pub struct Tag {
        name: String,
    }

    impl Tag {
        pub fn new(name: String) -> Self {
            Self { name }
        }

        pub fn get_name(&self) -> String {
            self.name.to_string()
        }

        pub fn rename(mut self, new_name: String) {
            self.name = new_name;
        }
    }
}

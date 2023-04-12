pub struct Text {
    pub id: String,
    pub name: String,
    pub text: String,
    pub ip: String,
}

impl Text {
    fn base_url(&self) -> String {
        return format!("http://{}:8088/api/?", self.ip);
    }
    fn default_select(&self) -> String {
        return format!("Input={}&SelectedName={}", &self.id, &self.name);
    }

    pub fn name_format(&mut self, iteration: u8, pre_name: &str) {
        self.name = format!("{}{}p4.Text", pre_name, iteration);
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        //reqwest::blocking::get(format!("{}Function=SetText&Value={}&{}", self.base_url(), self.text, self.default_select())).unwrap();
    }
    pub fn toggle_visibility(&mut self) {
        //reqwest::blocking::get(format!("{}Function=SetImageVisible&{}", self.base_url(), self.default_select())).unwrap();
        //reqwest::blocking::get(format!("{}Function=SetTextVisible&{}", self.base_url(), self.default_select())).unwrap();
    }
}

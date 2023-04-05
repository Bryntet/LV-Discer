use surf::Request;




pub struct Text {
    pub id: String,
    pub name: String,
    pub text: String,
}




impl Text {
    fn default_select(&self) -> String {
        return format!("Input={}&SelectedName={}", &self.id, &self.name);
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        let resp = reqwest::blocking::get(format!("http://192.168.120.109:8088/api/?Function=SetText&Value={}&{}", self.text, self.default_select())).unwrap();
        println!("{:#?}", resp);
    }

}

fn main() {
    let mut text = Text {
        id: String::from("909fecdd-3c51-4308-9a37-5365a1eb261c"),
        name: String::from("TextBlock3.Text"),
        text: String::from(""),
    };
    text.set_text(String::from("Hello World"));
}
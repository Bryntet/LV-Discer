pub enum Image {
    Nothing,
    GreenTriUp,
    RedTriDown,
    Flames,
}

impl Image {
    pub fn to_location(&self) -> String {
        String::from("C:\\livegrafik-flipup\\")
            + match self {
                Image::Nothing => "alpha.png",
                Image::GreenTriUp => "greentri.png",
                Image::RedTriDown => "redtri.png",
                Image::Flames => "fire.png",
            }
    }
}

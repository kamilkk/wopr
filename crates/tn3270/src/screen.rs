pub struct Field { pub row: u16, pub col: u16, pub protected: bool, pub hidden: bool, pub text: String }
pub struct Screen { pub fields: Vec<Field>, pub cursor: (u16, u16) }

impl Screen {
    /// A minimal logon panel: labels + userid + non-display password.
    pub fn logon() -> Self {
        Screen { fields: vec![
            Field { row: 2, col: 0,  protected: true,  hidden: false, text: "USERID   ===>".into() },
            Field { row: 2, col: 14, protected: false, hidden: false, text: String::new() },
            Field { row: 3, col: 0,  protected: true,  hidden: false, text: "PASSWORD ===>".into() },
            Field { row: 3, col: 14, protected: false, hidden: true,  text: String::new() },
        ], cursor: (2, 14) }
    }
}
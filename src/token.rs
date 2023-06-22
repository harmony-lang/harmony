#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl SourceLocation {
    pub fn merge(&self, other: &SourceLocation) -> SourceLocation {
        let mut location = self.clone();
        location.length = other.column + other.length - self.column;
        location
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.file.to_string(),
            self.line.to_string(),
            self.column.to_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharacterLiteral,
    BooleanLiteral,

    // Keywords
    Module,
    Import,
    As,
    Exposing,
    Enum,
    Fun,
    Case,
    Of,
    End,
    If,
    Then,
    Else,

    // Types
    Int,
    Float,
    String,
    Char,
    Bool,

    // Punctuation
    OpenParenthesis,  // (
    CloseParenthesis, // )
    OpenBracket,      // [
    CloseBracket,     // ]
    Dot,              // .
    DoubleDot,        // ..   (range)
    Comma,            // ,
    Colon,            // :
    DoubleColon,      // ::   (list constructor (cons))
    Pipe,             // |
    Arrow,            // ->
    FatArrow,         // =>

    // Operators
    Plus,              // +     (addition)
    PlusPlus,          // ++    (concatenation)
    Minus,             // -     (subtraction)
    Asterisk,          // *     (multiplication)
    Slash,             // /     (division)
    Percent,           // %     (modulo)
    Equals,            // =     (assignment)
    DoubleEquals,      // ==    (equality)
    NotEquals,         // /=    (inequality)
    LessThan,          // <     (less than)
    LessThanEquals,    // <=    (less than or equal to)
    GreaterThan,       // >     (greater than)
    GreaterThanEquals, // >=    (greater than or equal to)
    And,               // &&    (and)
    Or,                // ||    (or)
    Not,               // !     (not)

    // Special
    Newline,
    Whitespace,
    EndOfFile,
    Unknown,
}

impl TokenKind {
    pub fn is_binary_operator(&self) -> bool {
        match self {
            TokenKind::Plus
            | TokenKind::PlusPlus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::DoubleEquals
            | TokenKind::NotEquals
            | TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals
            | TokenKind::And
            | TokenKind::Or => true,
            _ => false,
        }
    }

    pub fn is_unary_operator(&self) -> bool {
        match self {
            TokenKind::Minus | TokenKind::Not => true,
            _ => false,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
            TokenKind::Or => 1,
            TokenKind::And => 2,
            TokenKind::DoubleEquals | TokenKind::NotEquals => 3,
            TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => 6,
            TokenKind::PlusPlus => 7,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
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
    IdentifierLiteral,
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
    EndOfFile,
    Unknown,
}

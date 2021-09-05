use super::super::located::Located;

// == structural ==
#[derive(Debug)]
pub enum ASTModule {
    Module { 
        items: Vec<Located<ASTItem>>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTItem {
    Def(ASTDef),
    View(ASTView),
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTDef {
    Def {
        name: Located<String>,
        args: Located<ASTArgs>,
        return_type: Option<Located<ASTTypes>>,
        body: Located<ASTBlock>
    },
    Invalid(KupoParseError),
}
#[derive(Debug)]
pub enum ASTView {
    View {
        name: Located<String>,
        args: Located<ASTArgs>,
        clauses: Vec<Located<ASTQueryExpression>>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTArgs {
    Args {
        args: Vec<Located<ASTArg>>,
    },
    Invalid(KupoParseError),
}


#[derive(Debug)]
pub enum ASTTypes {
    Types {
        types: Vec<Located<ASTType>>,
    },
    Invalid(KupoParseError),
}


#[derive(Debug)]
pub enum ASTArg {
    Arg {
        name: Located<String>,
        type_name: Option<Located<ASTType>>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTType {
    Type { name: Located<String>, },
    Invalid(KupoParseError),
}

// == statement ==
#[derive(Debug)]
pub enum ASTBlock {
    Block {
        items: Vec<Located<ASTStatement>>
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTStatement {
    For { 
        arg: Located<ASTQueryExpression>, 
        body: Located<ASTBlock>
    },
    If {
        arg: Located<ASTQueryExpression>,
        body: Located<ASTBlock>,
        else_: Option<Located<ASTBlock>>,
    },
    Return {
        arg: Located<ASTExpression>
    },
    Call { 
        call: Located<ASTCall>,
    },
    Assign {
        first: bool,  // := vs =
        variable: Located<ASTAssignTarget>,
        arg: Located<ASTExpression>,
    },
    Invalid(KupoParseError),
    // TODO: continue/break 
}

// == query expression ==
#[derive(Debug)]
pub enum ASTQueryExpression {
    QExpression { 
        items: Vec<Located<ASTQueryGoal>>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTQueryGoal {
    // TODO: Parens?
    Goal { 
        args: Located<ASTAssignTarget>,
        source: Located<ASTQueryGoalSource>,
    },
}

#[derive(Debug)]
pub enum ASTAssignTarget {
    Target {
        args: Vec<Located<ASTExpression>>,
    },
    Invalid(KupoParseError)
}

#[derive(Debug)]
pub enum ASTQueryGoalSource {
    In { from: Located<String>, },
    Assign { expression: Located<ASTExpression>, },
    // TODO: Also allow = instead of := but explain that it is wrong.
    // TODO: Allow arbitrary boolean expressions
    Invalid(KupoParseError),
}

// == expression ==
#[derive(Debug)]
pub enum ASTExpression {
    StringLiteral { it: String },
    IntegerLiteral { it: u64 },
    Call { 
        call: Located<ASTCall>,
    },
    UOp {
        op: ASTUOp,
        arg: Box<Located<ASTExpression>>
    },
    BinOp {
        arg1: Box<Located<ASTExpression>>,
        op: ASTBinOp,
        arg2: Box<Located<ASTExpression>>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTUOp { 
    Negate, Plus
}

#[derive(Debug)]
pub enum ASTBinOp { 
    Add, Subtract, Multiply, Divide
}

#[derive(Debug)]
pub enum ASTCall {
    Call {
        name: Located<String>,
        args: Located<ASTCallArgs>,
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub enum ASTCallArgs {
    Args {
        args: Vec<Located<ASTExpression>>
    },
    Invalid(KupoParseError),
}

#[derive(Debug)]
pub struct KupoParseError(pub String);
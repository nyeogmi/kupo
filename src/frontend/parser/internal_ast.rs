use super::super::located::Located;

pub struct ASTModule {
    pub items: Vec<Located<ASTItem>>,
}

pub enum ASTItem {
    Def(ASTDef),
    View(ASTView),
    Invalid(KupoParseError),
}

pub enum ASTDef {
    Def {
        name: Located<String>,
        args: Located<ASTArgs>,
        body: Located<ASTBlock>
    },
    Invalid(KupoParseError),
}

pub enum ASTArgs {
    Args {
        args: Vec<Located<ASTArg>>,
    },
    Invalid(KupoParseError),
}

pub enum ASTView {
    View {
        name: Located<String>,
        args: Located<ASTArgs>,
        clauses: Vec<Located<ASTQueryExpression>>,
    },
    Invalid(KupoParseError),
}

pub enum ASTArg {
    Arg {
        name: Located<String>,
        type_name: Option<Located<String>>,
    },
    Invalid(KupoParseError),
}

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
    // TODO: Return 
}

pub enum ASTBlock {
    Block {
        items: Vec<Located<ASTStatement>>
    },
    Invalid(KupoParseError),
}

pub enum ASTQueryExpression {
    QExpression { 
        items: Vec<Located<ASTQueryGoal>>,
    },
    Invalid(KupoParseError),
}

pub enum ASTQueryGoal {
    // TODO: Parens?
    Goal { 
        args: Located<ASTAssignTarget>,
        source: Located<ASTQueryGoalSource>,
    },
    Invalid(KupoParseError),
}

pub enum ASTAssignTarget {
    Target {
        args: Vec<Located<ASTExpression>>,
    },
    Invalid(KupoParseError)
}

pub enum ASTQueryGoalSource {
    In { from: Located<String>, },
    Assign { expression: Located<ASTExpression>, },
    // TODO: Also allow = instead of := but explain that it is wrong.
    // TODO: Allow arbitrary boolean expressions
    Invalid(KupoParseError),
}

pub enum ASTExpression {
    StringLiteral { it: Located<String> },
    Call { 
        call: Located<ASTCall>,
    },
    Invalid(KupoParseError),
}

pub enum ASTCall {
    Call {
        name: Located<String>,
        args: Located<ASTCallArgs>,
    },
    Invalid(KupoParseError),
}

pub enum ASTCallArgs {
    Args {
        args: Vec<Located<ASTExpression>>
    },
    Invalid(KupoParseError),
}

pub struct KupoParseError(pub String);

// NYEO NOTE: use this if the parse is so screwed up it can't even continue
// in particular, if you can't even tell what the location of the problem is
pub struct KupoParsePanic(pub String);
use super::located::Located;
use super::parser::internal_ast::{self, KupoParseError};

// NYEO NOTE: some of the internal_ast types have been flattened out
// because the only reason to give them their own AST level
// was that something was possibly wrong with them

// == structural ==
#[derive(Debug)]
pub struct Module {
    items: Vec<Located<Item>>
}

#[derive(Debug)]
pub enum Item {
    Def(Def),
    View(View),
}

#[derive(Debug)]
pub struct Def {
    name: Located<String>,
    args: Vec<Located<Arg>>,
    return_type: Option<Vec<Located<Type>>>,
    body: Located<Block>,
}

#[derive(Debug)]
pub struct View {
    name: Located<String>,
    args: Vec<Located<Arg>>,
    clauses: Vec<Located<QueryExpression>>
}

#[derive(Debug)]
pub struct Arg {
    name: Located<String>,
    type_name: Option<Located<Type>>,
}

#[derive(Debug)]
pub struct Type {
    name: Located<String>,
}

// == statement ==
#[derive(Debug)]
pub struct Block {
    items: Vec<Located<Statement>>,
}

#[derive(Debug)]
pub enum Statement {
    For {
        arg: Located<QueryExpression>,
        body: Located<Block>,
    },
    If { 
        arg: Located<QueryExpression>,
        body: Located<Block>,
        else_: Option<Located<Block>>,
    },
    Return { 
        arg: Located<Expression>,
    },
    Call {
        call: Located<Call>
    },
    Assign {
        first: bool, // := vs =
        variable: Located<AssignTarget>,

        arg: Located<Expression>,
    },
}

#[derive(Debug)]
pub struct AssignTarget {
    args: Vec<Located<Expression>>,
}

// == query expression ==
#[derive(Debug)]
pub struct QueryExpression {
    items: Vec<Located<QueryGoal>>
}

#[derive(Debug)]
pub enum QueryGoal {
    In {
        args: Located<AssignTarget>,
        from: Located<String>,
    },
    Assign {
        args: Located<AssignTarget>,
        expression: Located<Expression>
    },
}

// == expression ==
#[derive(Debug)]
pub enum Expression {
    StringLiteral { it: String },
    IntegerLiteral { it: u64 },
    Call { 
        call: Located<Call>
    },
    UOp {
        op: UOp,
        arg: Box<Located<Expression>>,
    },
    BinOp {
        arg1: Box<Located<Expression>>,
        op: BinOp,
        arg2: Box<Located<Expression>>,
    }
}

#[derive(Debug)]
pub enum UOp { Negate, Plus }

#[derive(Debug)]
pub enum BinOp { Add, Subtract, Multiply, Divide }

#[derive(Debug)]
pub struct Call {
    name: Located<String>,
    args: Vec<Located<Expression>>,
}

// == convert internal to external ast ==
pub type Errors = Vec<Located<KupoParseError>>;

pub enum Simp<T> {
    InProgress(T),
    Failed(Errors),
}

impl<T> Simp<T> {
    pub fn new(t: T) -> Simp<T> { Simp::InProgress(t) }
    pub fn fail(e: Located<KupoParseError>) -> Simp<T> { Simp::Failed(vec![e]) }

    pub fn concat(simps: impl Iterator<Item=Simp<T>>) -> Simp<Vec<T>> {
        let mut result = Simp::new(vec![]);
        for simp in simps {
            result.merge_mut(simp, |m, i| { m.push(i) });
        }
        result
    }

    pub fn simpmap<T2>(self, f: impl FnOnce(T) -> T2) -> Simp<T2> {
        match self {
            Simp::InProgress(t) => Simp::InProgress(f(t)),
            Simp::Failed(e) => Simp::Failed(e),
        }
    }

    pub fn new_with_optional<T2>(value: Option<T>, f: impl FnOnce(T) -> Simp<T2>) -> Simp<Option<T2>> {
        match value { 
            Some(t) => f(t).simpmap(Some),
            None => Simp::new(None),
        }
    }

    pub fn merge_mut<T2>(&mut self, s: Simp<T2>, merge: impl FnOnce(&mut T, T2)) {
        match (self, s) {
            (Simp::InProgress(t), Simp::InProgress(t2)) => {
                merge(t, t2);
            }
            (Simp::Failed(e), Simp::Failed(e2)) => {
                e.extend(e2);
            }
            (Simp::Failed(_), _) => { }
            (x, Simp::Failed(e)) => { 
                *x = Simp::Failed(e) 
            }
        };
    }

    pub fn merge<T2, T3>(self, s: Simp<T2>, merge: impl FnOnce(T, T2) -> T3) -> Simp<T3> {
        match (self, s) {
            (Simp::InProgress(t), Simp::InProgress(t2)) => Simp::InProgress(merge(t, t2)),
            (Simp::Failed(mut e), Simp::Failed(e2)) => {
                e.extend(e2);
                Simp::Failed(e)
            },
            (Simp::Failed(e), _) => Simp::Failed(e),
            (_, Simp::Failed(e)) => Simp::Failed(e),
        }
    }

    pub fn tup2<T2>(s: Simp<T>, s2: Simp<T2>) -> Simp<(T, T2)> {
        s.merge(s2, |x, y| (x, y))
    }

    pub fn tup3<T2, T3>(s: Simp<T>, s2: Simp<T2>, s3: Simp<T3>) -> Simp<(T, T2, T3)> {
        Self::tup2(s, s2).merge(s3, |(x, y), z| (x, y, z))
    }

    pub fn tup4<T2, T3, T4>(s: Simp<T>, s2: Simp<T2>, s3: Simp<T3>, s4: Simp<T4>) -> Simp<(T, T2, T3, T4)> {
        Self::tup3(s, s2, s3).merge(s4, |(x, y, z), w| (x, y, z, w))
    }

    pub fn to_result(self) -> Result<T, Errors> {
        match self {
            Simp::InProgress(t) => Ok(t),
            Simp::Failed(e) => Err(e),
        }
    }
}

pub fn simplify_module(it: Located<internal_ast::ASTModule>) -> Result<Located<Module>, Errors> {
    _simplify_module(it).to_result()
}

fn _simplify_module(it: Located<internal_ast::ASTModule>) -> Simp<Located<Module>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTModule::Module { items } => {
            Simp::concat(items.into_iter().map(_simplify_item)).simpmap(
                |items|
                loc.replace(Module { items })
            )
        }
        internal_ast::ASTModule::Invalid(e) => Simp::fail(loc.replace(e)) 
    }
}

fn _simplify_item(it: Located<internal_ast::ASTItem>) -> Simp<Located<Item>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTItem::Def(d) => 
            _simplify_def(loc.replace(d)).simpmap(|d| loc.replace(Item::Def(d.value))),
        internal_ast::ASTItem::View(v) => 
            _simplify_view(loc.replace(v)).simpmap(|v| loc.replace(Item::View(v.value))),
        internal_ast::ASTItem::Invalid(e) => 
            Simp::fail(loc.replace(e))
    }
}

fn _simplify_def(it: Located<internal_ast::ASTDef>) -> Simp<Located<Def>> { 
    let loc = it.location();
    match it.value {
        internal_ast::ASTDef::Def { name, args, return_type, body } => {
            Simp::tup4(
                Simp::new(name),
                _simplify_args(args),
                Simp::new_with_optional(return_type, _simplify_types),
                _simplify_block(body)
            ).simpmap(
                |(name, args, return_type, body)|
                loc.replace(Def { name, args, return_type, body})
            )
        }
        internal_ast::ASTDef::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_view(it: Located<internal_ast::ASTView>) -> Simp<Located<View>> { // not located; the enclosing item is
    let loc = it.location();
    match it.value {
        internal_ast::ASTView::View { name, args, clauses } => {
            Simp::tup3(
                Simp::new(name),
                _simplify_args(args),
                Simp::concat(clauses.into_iter().map(_simplify_query_expression))
            ).simpmap(
                |(name, args, clauses)|
                loc.replace(View { name, args, clauses})
            )
        }
        internal_ast::ASTView::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_args(it: Located<internal_ast::ASTArgs>) -> Simp<Vec<Located<Arg>>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTArgs::Args { args } => 
            Simp::concat(args.into_iter().map(_simplify_arg)),
        internal_ast::ASTArgs::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_types(it: Located<internal_ast::ASTTypes>) -> Simp<Vec<Located<Type>>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTTypes::Types { types } => 
            Simp::concat(types.into_iter().map(_simplify_type)),
        internal_ast::ASTTypes::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_arg(it: Located<internal_ast::ASTArg>) -> Simp<Located<Arg>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTArg::Arg { name, type_name } =>
            Simp::tup2(
                Simp::new(name), 
                Simp::new_with_optional(type_name, _simplify_type),
            ).simpmap(
                |(name, type_name)| loc.replace(Arg { name, type_name })
            ),
        internal_ast::ASTArg::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_type(it: Located<internal_ast::ASTType>) -> Simp<Located<Type>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTType::Type { name } => Simp::new(loc.replace(Type { name })),
        internal_ast::ASTType::Invalid(e) => Simp::fail(loc.replace(e)),
    }
}

// == statement ==
fn _simplify_block(it: Located<internal_ast::ASTBlock>) -> Simp<Located<Block>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTBlock::Block { items } => 
            Simp::concat(items.into_iter().map(_simplify_statement)).simpmap(
                |items| loc.replace(Block { items })
            ),
        internal_ast::ASTBlock::Invalid(e) => Simp::fail(loc.replace(e)),
    }
}

fn _simplify_statement(it: Located<internal_ast::ASTStatement>) -> Simp<Located<Statement>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTStatement::For { arg, body } => 
            Simp::tup2(_simplify_query_expression(arg), _simplify_block(body)).simpmap(
                |(arg, body)| loc.replace(Statement::For { arg, body })
            ),
        internal_ast::ASTStatement::If { arg, body, else_ } => 
            Simp::tup3(
                _simplify_query_expression(arg), 
                _simplify_block(body),
                Simp::new_with_optional(else_, _simplify_block),
            ).simpmap(
                |(arg, body, else_)|
                loc.replace(Statement::If { arg, body, else_})
            ),
        internal_ast::ASTStatement::Return { arg } =>
            _simplify_expression(arg).simpmap(|arg| 
                loc.replace(Statement::Return { arg })
            ),
        internal_ast::ASTStatement::Call { call } => 
            _simplify_call(call).simpmap(|call| 
                loc.replace(Statement::Call { call })
            ),
        internal_ast::ASTStatement::Assign { first, variable, arg } => 
            Simp::tup2(
                _simplify_assign_target(variable),
                _simplify_expression(arg),
            ).simpmap(
                |(variable, arg)|
                loc.replace(Statement::Assign { first, variable, arg })
            ),
        internal_ast::ASTStatement::Invalid(e) => Simp::fail(loc.replace(e)),
    }
}

// == query expression ==
fn _simplify_query_expression(it: Located<internal_ast::ASTQueryExpression>) -> Simp<Located<QueryExpression>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTQueryExpression::QExpression { items } => {
            Simp::concat(items.into_iter().map(_simplify_query_goal)).simpmap(
                |items| loc.replace(QueryExpression { items })
            )
        }
        internal_ast::ASTQueryExpression::Invalid(e) => Simp::fail(loc.replace(e))
    }
}

fn _simplify_query_goal(it: Located<internal_ast::ASTQueryGoal>) -> Simp<Located<QueryGoal>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTQueryGoal::Goal { 
            args, 
            source
        } => {
            let loc_source = source.location();
            match source.value {
                internal_ast::ASTQueryGoalSource::In { from } => 
                    Simp::tup2(
                        _simplify_assign_target(args),
                        Simp::new(from),
                    ).simpmap(|(args, from)| 
                        loc.replace(QueryGoal::In { args, from })
                    ),
                internal_ast::ASTQueryGoalSource::Assign { expression } => 
                    Simp::tup2(
                        _simplify_assign_target(args),
                        _simplify_expression(expression),
                    ).simpmap(|(args, expression)| 
                        loc.replace(QueryGoal::Assign { args, expression })
                    ),
                internal_ast::ASTQueryGoalSource::Invalid(e) => 
                    Simp::fail(loc_source.replace(e))
            }
        }
    }
}

fn _simplify_assign_target(it: Located<internal_ast::ASTAssignTarget>) -> Simp<Located<AssignTarget>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTAssignTarget::Target { args } => 
            Simp::concat(args.into_iter().map(_simplify_expression)).simpmap(
                |args| loc.replace(AssignTarget { args })
            ),
        internal_ast::ASTAssignTarget::Invalid(e) => 
            Simp::fail(loc.replace(e))
    }
}

// == expression ==
fn _simplify_expression(it: Located<internal_ast::ASTExpression>) -> Simp<Located<Expression>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTExpression::StringLiteral { it } => 
            Simp::new(loc.replace(Expression::StringLiteral { it })),
        internal_ast::ASTExpression::IntegerLiteral { it } => 
            Simp::new(loc.replace(Expression::IntegerLiteral { it })),
        internal_ast::ASTExpression::Call { call } => 
            _simplify_call(call).simpmap(|call| loc.replace(Expression::Call { call })),
        internal_ast::ASTExpression::UOp { op, arg } => 
            _simplify_expression(*arg).simpmap(|arg|
                loc.replace(Expression::UOp { 
                    op: _simplify_uop(op), 
                    arg: Box::new(arg) ,
                })
            ),
        internal_ast::ASTExpression::BinOp { arg1, op, arg2 } => 
            Simp::tup2(
                _simplify_expression(*arg1),
                _simplify_expression(*arg2),
            ).simpmap(|(arg1, arg2)|
                loc.replace(Expression::BinOp { 
                    arg1: Box::new(arg1),
                    op: _simplify_binop(op), 
                    arg2: Box::new(arg2),
                })
            ),
        internal_ast::ASTExpression::Invalid(e) => 
            Simp::fail(loc.replace(e)),
    }
}

fn _simplify_call(it: Located<internal_ast::ASTCall>) -> Simp<Located<Call>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTCall::Call { name, args } => 
            _simplify_callargs(args).simpmap(|args| 
                loc.replace(Call { name, args })
            ),
        internal_ast::ASTCall::Invalid(e) => Simp::fail(loc.replace(e)),
    }
}

fn _simplify_callargs(it: Located<internal_ast::ASTCallArgs>) -> Simp<Vec<Located<Expression>>> {
    let loc = it.location();
    match it.value {
        internal_ast::ASTCallArgs::Args { args } => 
            Simp::concat(args.into_iter().map(_simplify_expression)),
        internal_ast::ASTCallArgs::Invalid(e) => Simp::fail(loc.replace(e)),
    }
}

fn _simplify_uop(it: internal_ast::ASTUOp) -> UOp {
    match it {
        internal_ast::ASTUOp::Negate => UOp::Negate,
        internal_ast::ASTUOp::Plus => UOp::Plus,
    }
}

fn _simplify_binop(it: internal_ast::ASTBinOp) -> BinOp {
    match it {
        internal_ast::ASTBinOp::Add => BinOp::Add,
        internal_ast::ASTBinOp::Subtract => BinOp::Subtract,
        internal_ast::ASTBinOp::Multiply => BinOp::Multiply,
        internal_ast::ASTBinOp::Divide => BinOp::Divide,
    }
}
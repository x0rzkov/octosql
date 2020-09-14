use crate::physical::physical::{Identifier, ScalarValue};
use crate::logical::logical::{Node, Expression, Aggregate};
use crate::parser;
use crate::parser::{identifier, Value, Source, Operator};
use datafusion::logicalplan::FunctionType::Scalar;
use std::collections::BTreeMap;
use crate::parser::parser::parse_sql;

pub fn query_to_logical_plan(query: &parser::Query) -> Box<Node> {
    match query {
        parser::Query::Select { expressions, filter, from, order_by } => {
            let mut plan = source_to_logical_plan(from.as_ref());

            let mut variables: BTreeMap<Identifier, Box<Expression>> = BTreeMap::new();

            let mut topmost_map_fields = Vec::with_capacity(expressions.len());

            for (i, (expr, alias)) in expressions.iter().enumerate() {
                let name = alias.clone()
                    .unwrap_or_else(|| if let parser::Expression::Variable(ident) = expr.as_ref() {
                        ident.clone()
                    } else {
                        parser::Identifier::SimpleIdentifier(format!("column_{}", i))
                    });
                let ident = identifier_to_logical_plan(&name);
                variables.insert(ident.clone(), expression_to_logical_plan(expr.as_ref()));

                topmost_map_fields.push(ident);
            }

            let bottom_map_expressions = variables.into_iter()
                .map(|(ident, expr)| {
                    (expr, ident)
                }).collect();

            plan = Box::new(Node::Map {
                source: plan,
                expressions: bottom_map_expressions,
                keep_source_fields: true,
            });

            if let Some(expr) = filter {
                plan = Box::new(Node::Filter { source: plan, filter_expr: expression_to_logical_plan(expr.as_ref()) });
            }

            let topmost_map_expressions = topmost_map_fields.into_iter()
                .map(|ident| (Box::new(Expression::Variable(ident.clone())), ident))
                .collect();

            plan = Box::new(Node::Map {
                source: plan,
                expressions: topmost_map_expressions,
                keep_source_fields: false,
            });

            plan
        }
    }
}

pub fn source_to_logical_plan(expr: &parser::Source) -> Box<Node> {
    match expr {
        parser::Source::Table(ident, alias) => {
            Box::new(Node::Source { name: identifier_to_logical_plan(&ident), alias: alias.clone().map(|ident| identifier_to_logical_plan(&ident)) })
        }
        parser::Source::Subquery(subquery, alias) => {
            query_to_logical_plan(&subquery)
        }
    }
}

pub fn expression_to_logical_plan(expr: &parser::Expression) -> Box<Expression> {
    match expr {
        parser::Expression::Variable(ident) => {
            Box::new(Expression::Variable(identifier_to_logical_plan(&ident)))
        }
        parser::Expression::Constant(value) => {
            Box::new(Expression::Constant(value_to_logical_plan(&value)))
        }
        parser::Expression::Function(name, args) => {
            unimplemented!()
        }
        parser::Expression::Operator(left, op, right) => {
            Box::new(Expression::Function(operator_to_logical_plan(op), vec![expression_to_logical_plan(left.as_ref()), expression_to_logical_plan(right.as_ref())]))
        }
        parser::Expression::Wildcard => {
            unimplemented!()
        }
    }
}

// TODO: Maybe it should be Aggregate(Expr), this way the aggregate receives the record batch and calculates everything itself.
// Would be easier for stars and stuff I suppose.
// Think about it.
// The Cons is that each aggregate will have to define evaluating the underlying expression, which might be meh.
// Especially since star and star distinct can operate on some kind of tuple... maybe?
pub fn aggregate_expression_to_logical_plan(expr: &parser::Expression) -> Box<Expression> {
    match expr {
        parser::Expression::Variable(ident) => {
            // Aggregate Expression :: Key Part With Name
            Box::new(Expression::Variable(identifier_to_logical_plan(&ident)))
        }
        parser::Expression::Function(name, args) => {
            // Aggregate Expression :: Aggregate
            unimplemented!()
        }
        parser::Expression::Wildcard => {
            unimplemented!()
        }
        _ => {
            dbg!(expr);
            panic!("invalid aggregate expression")
        }
    }
}


pub fn identifier_to_logical_plan(ident: & parser::Identifier) -> Identifier {
match ident {
parser::Identifier::SimpleIdentifier(id) => {
Identifier::SimpleIdentifier(id.clone())
}
parser::Identifier::NamespacedIdentifier(namespace, id) => {
Identifier::NamespacedIdentifier(namespace.clone(), id.clone())
}
}
}

pub fn value_to_logical_plan(val: & parser::Value) -> ScalarValue {
match val {
parser::Value::Integer(v) => {
ScalarValue::Int64(v.clone())
}
}
}

pub fn operator_to_logical_plan(op: & parser::Operator) -> Identifier {
Identifier::SimpleIdentifier( match op {
Operator::Eq => "=".to_string(),
Operator::Plus => "+".to_string(),
Operator::Minus => "-".to_string(),
Operator::AND => "AND".to_string(),
Operator::OR => "OR".to_string(),
})
}

# [test]
fn my_test() {
let sql = "SELECT c.name, COUNT(*), SUM(c.livesleft) \
    FROM cats c \
    WHERE c.age = c.livesleft \
    GROUP BY c.name";

let query = parse_sql(sql);
let plan = query_to_logical_plan(query.as_ref());
dbg ! (plan);
}

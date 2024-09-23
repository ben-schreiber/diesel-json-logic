#[macro_export]
macro_rules! unpack_json_logic_query {
    (
        [$($column:ident),*],
        $query:ident,
        $from_stmt:ident
    ) => {{
        $(
            if let Some(logic_query) = $query.$column {
                $from_stmt = match logic_query {
                    $crate::JsonLogicExpr::Eq(var, value) => match value {
                        Some(value) => $from_stmt.filter(diesel::ExpressionMethods::eq(var.to_diesel_column(), value)),
                        None => $from_stmt.filter(diesel::ExpressionMethods::is_null(var.to_diesel_column())),
                    },
                    $crate::JsonLogicExpr::Gt(var, value) => {
                        $from_stmt.filter(diesel::ExpressionMethods::gt(var.to_diesel_column(), value))
                    }
                    $crate::JsonLogicExpr::Lt(var, value) => {
                        $from_stmt.filter(diesel::ExpressionMethods::lt(var.to_diesel_column(), value))
                    }
                    $crate::JsonLogicExpr::In(var, values) => {
                        $from_stmt.filter(diesel::ExpressionMethods::eq_any(var.to_diesel_column(), values))
                    }
                };
            }
        )*
        $from_stmt
    }};
}

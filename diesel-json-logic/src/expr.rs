#[derive(Debug, ::serde::Deserialize, PartialEq)]
pub enum JsonLogicExpr<Column, T> {
    #[serde(rename = "==")]
    Eq(Column, Option<T>),
    #[serde(rename = "<")]
    Lt(Column, T),
    #[serde(rename = ">")]
    Gt(Column, T),
    #[serde(rename = "in")]
    In(Column, Vec<T>),
}

#[cfg(test)]
mod tests {
    use super::JsonLogicExpr;
    use chrono::prelude::*;
    use diesel::{debug_query, pg::Pg, QueryDsl};
    use diesel_json_logic_macros::define_json_logic;

    diesel::table! {
        tbl_one (id) {
            id -> Int4,
            created_at -> Timestamptz,
            notes -> VarChar,
        }
    }

    diesel::table! {
        tbl_two (id) {
            id -> Int4,
            created_at -> Timestamptz,
            other_notes -> VarChar,
        }
    }

    diesel::joinable!(
        tbl_one -> tbl_two (id)
    );

    diesel::allow_tables_to_appear_in_same_query!(tbl_one, tbl_two);

    #[test]
    #[allow(unused_macros)]
    fn test_logic_from_str() {
        define_json_logic!(
            TwoTablesQuery,
            [
                #[diesel_column_name = tbl_one::id]
                tbl_id => i32,
                #[diesel_column_name = tbl_two::created_at]
                tbl_two_created_at => DateTime<Utc>,
                #[diesel_column_name = tbl_two::other_notes]
                tbl_two_other_notes => String,
            ],
            true,
        );

        let result: JsonLogicExpr<TblIdVar, i32> =
            serde_json::from_str(r#"{"<": [{"var": "tbl_id"}, 1]}"#).unwrap();
        assert_eq!(
            result,
            JsonLogicExpr::Lt(
                TblIdVar {
                    var: TblId("tbl_id".to_string())
                },
                1
            )
        );

        let result: JsonLogicExpr<TblIdVar, i32> =
            serde_json::from_str(r#"{">": [{"var": "tbl_id"}, 1]}"#).unwrap();
        assert_eq!(
            result,
            JsonLogicExpr::Gt(
                TblIdVar {
                    var: TblId("tbl_id".to_string())
                },
                1
            )
        );

        let result: JsonLogicExpr<TblTwoCreatedAtVar, DateTime<Utc>> =
            serde_json::from_str(r#"{"==": [{"var": "tbl_two_created_at"}, null]}"#).unwrap();
        assert_eq!(
            result,
            JsonLogicExpr::Eq(
                TblTwoCreatedAtVar {
                    var: TblTwoCreatedAt("tbl_two_created_at".to_string())
                },
                None
            )
        );

        let result: JsonLogicExpr<TblTwoOtherNotesVar, String> =
            serde_json::from_str(r#"{"in": [{"var": "tbl_two_other_notes"}, ["a", "b"]]}"#)
                .unwrap();
        assert_eq!(
            result,
            JsonLogicExpr::In(
                TblTwoOtherNotesVar {
                    var: TblTwoOtherNotes("tbl_two_other_notes".to_string())
                },
                vec!["a".to_string(), "b".to_string()],
            )
        );
    }

    #[test]
    fn test_generate_sql_filters() {
        define_json_logic!(
            TwoTablesQuery,
            [
                #[diesel_column_name = tbl_one::id]
                tbl_id => i32,
                #[diesel_column_name = tbl_two::created_at]
                tbl_two_created_at => DateTime<Utc>,
                #[diesel_column_name = tbl_two::other_notes]
                tbl_two_other_notes => String,
            ],
            true,
        );

        let json_logic_one = serde_json::from_str(r#"{"<": [{"var": "tbl_id"}, 1]}"#).unwrap();
        let query = TwoTablesQuery {
            tbl_id: Some(json_logic_one),
            tbl_two_created_at: None,
            tbl_two_other_notes: None,
        };

        let mut select_stmt = tbl_one::table.inner_join(tbl_two::table).into_boxed();
        select_stmt = impl_two_tables_query!(query, select_stmt);

        let generated_sql = debug_query::<Pg, _>(&select_stmt).to_string();
        let expected_sql = r#"SELECT "tbl_one"."id", "tbl_one"."created_at", "tbl_one"."notes", "tbl_two"."id", "tbl_two"."created_at", "tbl_two"."other_notes" FROM ("tbl_one" INNER JOIN "tbl_two" ON ("tbl_one"."id" = "tbl_two"."id")) WHERE ("tbl_one"."id" < $1) -- binds: [1]"#;
        assert_eq!(generated_sql, expected_sql);
    }
}

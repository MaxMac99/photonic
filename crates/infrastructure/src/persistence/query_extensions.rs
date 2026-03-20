// use sqlx::{Database, Encode, QueryBuilder, Type};
//
// pub trait DirectionExt {
//     fn to_sql(&self) -> &'static str;
// }
//
// impl DirectionExt for Direction {
//     fn to_sql(&self) -> &'static str {
//         match self {
//             Direction::Asc => "ASC",
//             Direction::Desc => "DESC",
//         }
//     }
// }
//
// pub trait UpdateOptionalQueryExt<T> {
//     fn apply_to_query<'a, DB: Database>(
//         &'a self,
//         query: &mut QueryBuilder<'a, DB>,
//         column_name: &str,
//     ) where
//         T: 'a + Send + Encode<'a, DB> + Type<DB>;
//
//     fn apply_to_separated<'a, DB: Database>(
//         &'a self,
//         separated: &mut sqlx::query_builder::Separated<'a, 'a, DB, &'static str>,
//         column_name: &str,
//     ) where
//         T: 'a + Send + Encode<'a, DB> + Type<DB>;
// }
//
// impl<T> UpdateOptionalQueryExt<T> for UpdateOptional<T> {
//     fn apply_to_query<'a, DB: Database>(
//         &'a self,
//         query: &mut QueryBuilder<'a, DB>,
//         column_name: &str,
//     ) where
//         T: 'a + Send + Encode<'a, DB> + Type<DB>,
//     {
//         match self {
//             UpdateOptional::Ignore => {}
//             UpdateOptional::Clear => {
//                 query.push(column_name).push(" = NULL");
//             }
//             UpdateOptional::SetIfNull(value) => {
//                 query
//                     .push(column_name)
//                     .push(" = COALESCE(")
//                     .push(column_name)
//                     .push(", ")
//                     .push_bind(value)
//                     .push(")");
//             }
//             UpdateOptional::Replace(value) => {
//                 query.push(column_name).push(" = ").push_bind(value);
//             }
//         }
//     }
//
//     fn apply_to_separated<'a, DB: Database>(
//         &'a self,
//         separated: &mut sqlx::query_builder::Separated<'a, 'a, DB, &'static str>,
//         column_name: &str,
//     ) where
//         T: 'a + Send + Encode<'a, DB> + Type<DB>,
//     {
//         match self {
//             UpdateOptional::Ignore => {}
//             UpdateOptional::Clear => {
//                 separated.push(format!("{} = NULL", column_name));
//             }
//             UpdateOptional::SetIfNull(value) => {
//                 separated.push(column_name);
//                 separated.push_unseparated(" = COALESCE(");
//                 separated.push_unseparated(column_name);
//                 separated.push_unseparated(", ");
//                 separated.push_bind_unseparated(value);
//                 separated.push_unseparated(")");
//             }
//             UpdateOptional::Replace(value) => {
//                 separated.push(column_name);
//                 separated.push_unseparated(" = ");
//                 separated.push_bind_unseparated(value);
//             }
//         }
//     }
// }

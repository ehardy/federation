use graphql_parser::query::*;
use graphql_parser::schema::{InterfaceType, ObjectType, TypeDefinition, UnionType};
use graphql_parser::{query, schema, Name};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPlanningContext<'q, 's: 'q> {
    pub schema: &'s schema::Document<'s>,
    pub operation: &'q OperationDefinition<'q>,
    pub fragments: HashMap<&'q str, &'q FragmentDefinition<'q>>,
    pub possible_types: HashMap<&'s str, Vec<&'s schema::ObjectType<'s>>>,
    pub names_to_types: HashMap<&'s str, &'s TypeDefinition<'s>>,
    pub auto_fragmentization: bool,
}

impl<'q, 's: 'q> QueryPlanningContext<'q, 's> {
    pub fn new_scope(
        &self,
        td: &'s TypeDefinition<'s>,
        enclosing_scope: Option<&'q Scope<'q>>,
    ) -> Scope<'q> {
        let parent_possible_types = self.get_possible_types(td);

        let possible_types = match enclosing_scope {
            Some(enclosing_scope) => parent_possible_types
                .into_iter()
                .filter(|t| enclosing_scope.possible_types.contains(t))
                .collect(),
            None => parent_possible_types,
        };

        Scope {
            parent_type: GraphQLCompositeType::from(td),
            possible_types,
            enclosing_scope,
        }
    }

    pub fn get_type(&self, type_name: &str) -> &TypeDefinition {
        self.names_to_types[type_name]
    }

    fn get_possible_types(&self, td: &'s TypeDefinition<'s>) -> Vec<&'s schema::ObjectType<'s>> {
        self.possible_types[td.name().unwrap()].clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope<'q> {
    parent_type: GraphQLCompositeType<'q>,
    possible_types: Vec<&'q schema::ObjectType<'q>>,
    enclosing_scope: Option<&'q Scope<'q>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphQLCompositeType<'q> {
    Object(&'q ObjectType<'q>),
    Interface(&'q InterfaceType<'q>),
    Union(&'q UnionType<'q>),
}

impl<'q> From<&'q TypeDefinition<'q>> for GraphQLCompositeType<'q> {
    fn from(td: &'q TypeDefinition<'q>) -> Self {
        match td {
            TypeDefinition::Object(o) => GraphQLCompositeType::Object(o),
            TypeDefinition::Interface(iface) => GraphQLCompositeType::Interface(iface),
            TypeDefinition::Union(un) => GraphQLCompositeType::Union(un),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'q> {
    scope: &'q Scope<'q>,
    field_node: &'q query::Field<'q>,
    field_def: &'q schema::Field<'q>,
}

pub type FieldSet<'q> = Vec<Field<'q>>;

#[derive(Debug, Clone)]
pub struct FetchGroup<'a> {
    service: String,
    fields: FieldSet<'a>,
    internal_fragments: HashSet<&'a FragmentDefinition<'a>>,
}
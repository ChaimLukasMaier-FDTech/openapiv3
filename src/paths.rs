use std::marker::PhantomData;

use crate::*;
use indexmap::IndexMap;
use http::Method;
use serde::{Deserialize, Deserializer, Serialize};

/// Describes the operations available on a single path.
/// A Path Item MAY be empty, due to ACL constraints.
/// The path itself is still exposed to the documentation
/// viewer but they will not know which operations and
/// parameters are available.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PathItem {
    /// An optional, string summary, intended to apply to all operations in
    /// this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// An optional, string description, intended to apply to all operations in
    /// this path. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    /// An alternative server array to service all operations in this path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    /// A list of parameters that are applicable for all the
    /// operations described under this path. These parameters
    /// can be overridden at the operation level, but cannot be
    /// removed there. The list MUST NOT include duplicated parameters.
    /// A unique parameter is defined by a combination of a name and location.
    /// The list can use the Reference Object to link to parameters that
    /// are defined at the OpenAPI Object's components/parameters.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<RefOr<Parameter>>,
    /// Inline extensions to this object.
    #[serde(flatten, deserialize_with = "crate::util::deserialize_extensions")]
    pub extensions: IndexMap<String, serde_json::Value>,
}

impl PathItem {
    /// Returns an iterator of references to the [Operation]s in the [PathItem].
    pub fn iter(&self) -> impl Iterator<Item=(&str, &'_ Operation)> {
        vec![
            ("get", &self.get),
            ("put", &self.put),
            ("post", &self.post),
            ("delete", &self.delete),
            ("options", &self.options),
            ("head", &self.head),
            ("patch", &self.patch),
            ("trace", &self.trace),
        ]
            .into_iter()
            .filter_map(|(method, maybe_op)| maybe_op.as_ref().map(|op| (method, op)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(&str, &'_ mut Operation)> {
        vec![
            ("get", &mut self.get),
            ("put", &mut self.put),
            ("post", &mut self.post),
            ("delete", &mut self.delete),
            ("options", &mut self.options),
            ("head", &mut self.head),
            ("patch", &mut self.patch),
            ("trace", &mut self.trace),
        ]
            .into_iter()
            .filter_map(|(method, maybe_op)| maybe_op.as_mut().map(|op| (method, op)))
    }

    pub fn get(operation: Operation) -> Self {
        Self {
            get: Some(operation),
            ..PathItem::default()
        }
    }

    pub fn post(operation: Operation) -> Self {
        Self {
            post: Some(operation),
            ..PathItem::default()
        }
    }
}

impl IntoIterator for PathItem {
    type Item = (&'static str, Operation);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    /// Returns an iterator of the [Operation]s in the [PathItem].
    fn into_iter(self) -> Self::IntoIter {
        vec![
            ("get", self.get),
            ("put", self.put),
            ("post", self.post),
            ("delete", self.delete),
            ("options", self.options),
            ("head", self.head),
            ("patch", self.patch),
            ("trace", self.trace),
        ]
            .into_iter()
            .filter_map(|(method, maybe_op)| maybe_op.map(|op| (method, op)))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// Holds the relative paths to the individual endpoints and
/// their operations. The path is appended to the URL from the
/// Server Object in order to construct the full URL. The Paths
/// MAY be empty, due to ACL constraints.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Paths {
    /// A map of PathItems or references to them.
    #[serde(flatten, deserialize_with = "deserialize_paths")]
    pub paths: IndexMap<String, RefOr<PathItem>>,
    /// Inline extensions to this object.
    #[serde(flatten, deserialize_with = "crate::util::deserialize_extensions")]
    pub extensions: IndexMap<String, serde_json::Value>,
}

impl std::ops::Deref for Paths {
    type Target = IndexMap<String, RefOr<PathItem>>;

    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl std::ops::DerefMut for Paths {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.paths
    }
}

impl Paths {
    pub fn insert(&mut self, key: String, path_item: PathItem) -> Option<RefOr<PathItem>> {
        self.paths.insert(key, RefOr::Item(path_item))
    }

    pub fn insert_operation(&mut self, path: String, method: Method, operation: Operation) -> Option<Operation> {
        let item = self.paths.entry(path).or_default();
        let item = item.as_mut().expect("Currently don't support references for PathItem");
        match method {
            Method::GET => item.get.replace(operation),
            Method::PUT => item.put.replace(operation),
            Method::POST => item.post.replace(operation),
            Method::DELETE => item.delete.replace(operation),
            Method::PATCH => item.patch.replace(operation),
            Method::HEAD => item.head.replace(operation),
            Method::OPTIONS => item.options.replace(operation),
            Method::TRACE => item.trace.replace(operation),
            _ => panic!("Unsupported method: {:?}", method),
        }
    }
}

impl IntoIterator for Paths {
    type Item = (String, RefOr<PathItem>);

    type IntoIter = indexmap::map::IntoIter<String, RefOr<PathItem>>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.into_iter()
    }
}

fn deserialize_paths<'de, D>(
    deserializer: D,
) -> Result<IndexMap<String, RefOr<PathItem>>, D::Error>
    where
        D: Deserializer<'de>,
{
    deserializer.deserialize_map(PredicateVisitor(
        |key: &String| key.starts_with('/'),
        PhantomData,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_item_iterators() {
        let operation = Operation::default();

        let path_item = PathItem {
            get: Some(operation.clone()),
            post: Some(operation.clone()),
            delete: Some(operation.clone()),
            ..Default::default()
        };

        let expected = vec![
            ("get", &operation),
            ("post", &operation),
            ("delete", &operation),
        ];
        assert_eq!(path_item.iter().collect::<Vec<_>>(), expected);

        let expected = vec![
            ("get", operation.clone()),
            ("post", operation.clone()),
            ("delete", operation.clone()),
        ];
        assert_eq!(path_item.into_iter().collect::<Vec<_>>(), expected);
    }
}

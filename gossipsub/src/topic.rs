use base64::prelude::*;
use prometheus_client::encoding::EncodeLabelSet;
use quick_protobuf::Writer;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A generic trait that can be extended for various hashing types for a topic.
pub trait Hasher {
    /// The function that takes a topic string and creates a topic hash.
    fn hash(topic_string: String) -> TopicHash;
}

/// A type for representing topics who use the identity hash.
#[derive(Debug, Clone)]
pub struct IdentityHash {}
impl Hasher for IdentityHash {
    /// Creates a [`TopicHash`] as a raw string.
    fn hash(topic_string: String) -> TopicHash {
        TopicHash { hash: topic_string }
    }
}

#[derive(Debug, Clone)]
pub struct Sha256Hash {}
impl Hasher for Sha256Hash {
    /// Creates a [`TopicHash`] by SHA256 hashing the topic then base64 encoding the
    /// hash.
    fn hash(topic_string: String) -> TopicHash {
        use quick_protobuf::MessageWrite;

        let topic_descripter = proto::TopicDescriptor {
            name: Some(topic_string),
            auth: None,
            enc: None,
        };
        let mut bytes = Vec::with_capacity(topic_descripter.get_size());
        let mut writer = Writer::new(&mut bytes);
        topic_descripter
            .write_message(&mut writer)
            .expect("Encoding to succeed");
        let hash = BASE64_STANDARD.encode(Sha256::digest(&bytes));
        TopicHash { hash }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, EncodeLabelSet, Serialize, Deserialize)]
pub struct TopicHash {
    /// The topic hash. Stored as a string to align with the protobuf API.
    hash: String,
}

impl TopicHash {
    pub fn from_raw(hash: impl Into<String>) -> TopicHash {
        TopicHash { hash: hash.into() }
    }

    pub fn into_string(self) -> String {
        self.hash
    }

    pub fn as_str(&self) -> &str {
        &self.hash
    }
}

/// A gossipsub topic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Topic<H: Hasher> {
    topic: String,
    phantom_data: std::marker::PhantomData<H>,
}

impl<H: Hasher> From<Topic<H>> for TopicHash {
    fn from(topic: Topic<H>) -> TopicHash {
        topic.hash()
    }
}

impl<H: Hasher> Topic<H> {
    pub fn new(topic: impl Into<String>) -> Self {
        Topic {
            topic: topic.into(),
            phantom_data: std::marker::PhantomData,
        }
    }

    pub fn hash(&self) -> TopicHash {
        H::hash(self.topic.clone())
    }
}

impl<H: Hasher> fmt::Display for Topic<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.topic)
    }
}

impl fmt::Display for TopicHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hash)
    }
}

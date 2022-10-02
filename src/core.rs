use crate::extended::{Actor, ActorBuilder};
use crate::Serde;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// [Null]-type object that implements [Serde] for convenience
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Null {}

impl Serde<'_> for Null {}

// TODO: rename to something else as there's a [Document] in the Activity
// Streams spec.
/// Outer object for serialization and deserialization. Not an Activity Streams
/// 2.0 object.
#[derive(Serialize, Deserialize, Debug)]
pub struct Document<'a, T> {
    #[serde(rename = "@context", borrow)]
    pub context: Context<'a>,

    #[serde(flatten)]
    pub object: T,
}

impl<'de: 'a, 'a, T> Serde<'de> for Document<'a, T> where T: Serde<'de> {}

impl<'a, T: Serde<'a>> Document<'a, T> {
    pub fn new(context: Context<'a>, object: T) -> Self {
        Document { context, object }
    }
}

/// JSON-LD uses the special @context property to define the processing context.
/// The value of the @context property is defined by the [JSON-LD]
/// specification. Implementations producing Activity Streams 2.0 documents
/// should include a @context property with a value that includes a reference to
/// the normative Activity Streams 2.0 JSON-LD @context definition using the URL
/// "https://www.w3.org/ns/activitystreams". Implementations may use the
/// alternative URL "http://www.w3.org/ns/activitystreams" instead. This can be
/// done using a string, object, or array.
/// <https://www.w3.org/TR/activitystreams-core/#jsonld>
#[derive(Serialize, Deserialize, Debug)]
pub struct Context<'a> {
    #[serde(rename = "@vocab")]
    namespace: &'a str,

    #[serde(skip_serializing_if = "Option::is_none", rename = "@language")]
    language: Option<&'a str>,
}

/// Builder struct for [Context].
pub struct ContextBuilder<'a> {
    namespace: &'a str,
    language: Option<&'a str>,
}

impl<'a> ContextBuilder<'a> {
    const NAMESPACE: &'static str = "https://www.w3.org/ns/activitystreams";

    pub fn new() -> Self {
        ContextBuilder {
            namespace: ContextBuilder::NAMESPACE,
            language: None,
        }
    }

    // TODO: extend this to other options per the docs
    pub fn language(mut self, language: &'a str) -> Self {
        self.language = Some(language);
        self
    }

    pub fn build(self) -> Context<'a> {
        Context {
            namespace: self.namespace,
            language: self.language,
        }
    }
}

impl<'a> Default for ContextBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// The [Object] is the primary base type for the Activity Streams vocabulary.
/// In addition to having a global identifier (expressed as an absolute IRI
/// using the id property) and an "object type" (expressed using the type
/// property), all instances of the Object type share a common set of
/// properties normatively defined by the Activity Vocabulary. These
/// include: attachment | attributedTo | audience | content | context |
/// contentMap | name | nameMap | endTime | generator | icon | image |
/// inReplyTo | location | preview | published | replies | startTime |
/// summary | summaryMap | tag | updated | url | to | bto | cc | bcc |
/// mediaType | duration
/// All properties are optional (including the id and type).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Object<'a, AttributedToT> {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub image: Option<Box<Link<'a>>>,

    #[serde(
        rename = "attributedTo",
        skip_serializing_if = "Vec::is_empty",
        default = "Vec::new"
    )]
    pub attributed_to: Vec<AttributedToT>,

    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub audience: Option<Box<Object<'a, Null>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<&'a str>,
}

impl<'de: 'a, 'a, AttributedToT> Serde<'de> for Object<'a, AttributedToT> where
    AttributedToT: Serde<'de> + Clone
{
}

/// Builder for [Object].
#[derive(Clone)]
pub struct ObjectBuilder<'a, AttributedToT> {
    object_type: Option<&'a str>,
    // TODO: actually an IRI: consider https://docs.rs/iref/latest/iref/
    id: Option<http::Uri>,
    name: Option<&'a str>,
    url: Option<http::Uri>,
    published: Option<DateTime<Utc>>,
    image: Option<LinkBuilder<'a>>,
    attributed_to: Vec<AttributedToT>,
    audience: Option<Box<ObjectBuilder<'a, Null>>>,
    content: Option<&'a str>,
    summary: Option<&'a str>,
    // TODO: more fields
}

impl<'a, AttributedToT: Serde<'a> + Clone> ObjectBuilder<'a, AttributedToT> {
    pub fn new() -> Self {
        ObjectBuilder {
            object_type: None,
            id: None,
            name: None,
            url: None,
            published: None,
            image: None,
            attributed_to: vec![],
            audience: None,
            content: None,
            summary: None,
        }
    }

    pub fn object_type(mut self, object_type: &'a str) -> Self {
        self.object_type = Some(object_type);
        self
    }

    pub fn id(&mut self, id: http::Uri) -> Self {
        self.id = Some(id);
        self.clone()
    }

    pub fn name(&mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self.clone()
    }

    pub fn url(&mut self, url: http::Uri) -> Self {
        self.url = Some(url);
        self.clone()
    }

    pub fn published(&mut self, datetime: DateTime<Utc>) -> Self {
        self.published = Some(datetime);
        self.clone()
    }

    pub fn image(&mut self, image: LinkBuilder<'a>) -> Self {
        self.image = Some(image);
        self.clone()
    }

    pub fn add_attributed_to(mut self, attribution: AttributedToT) -> Self {
        self.attributed_to.push(attribution);
        self
    }

    pub fn audience(&mut self, audience: ObjectBuilder<'a, Null>) -> Self {
        self.audience = Some(Box::new(audience));
        self.clone()
    }

    pub fn content(mut self, content: &'a str) -> Self {
        self.content = Some(content);
        self
    }

    pub fn summary(&mut self, summary: &'a str) -> Self {
        self.summary = Some(summary);
        self.clone()
    }

    pub fn build(self) -> Object<'a, AttributedToT> {
        Object {
            object_type: self.object_type,
            id: self.id.map(|uri| uri.to_string()),
            name: self.name,
            url: self.url.map(|uri| uri.to_string()),
            published: self.published,
            image: self.image.map(|i| Box::new(i.build())),
            attributed_to: self.attributed_to,
            audience: self.audience.map(|a| Box::new(a.build())),
            content: self.content,
            summary: self.summary,
        }
    }
}

impl<'a, AttributedToT: Serde<'a> + Clone> Default for ObjectBuilder<'a, AttributedToT> {
    fn default() -> Self {
        Self::new()
    }
}

/// A utility struct to describe a URI.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Uri<'a> {
    pub href: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mediaType")]
    pub media_type: Option<&'a str>,
}

impl<'de: 'a, 'a> Serde<'de> for Uri<'a> {}

/// Builder struct for [Uri].
#[derive(Clone)]
pub struct UriBuilder<'a> {
    href: http::Uri,
    media_type: Option<&'a str>,
}

impl<'a> UriBuilder<'a> {
    pub fn new(href: http::Uri) -> Self {
        UriBuilder {
            href,
            media_type: None,
        }
    }

    pub fn media_type(mut self, media_type: &'a str) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn build(self) -> Uri<'a> {
        Uri {
            href: self.href.to_string(),
            media_type: self.media_type,
        }
    }
}

/// Identifies an entity that provides a preview of this object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preview<'a> {
    #[serde(flatten)]
    pub base: Object<'a, Null>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub url: Option<Uri<'a>>,
}

impl<'de: 'a, 'a> Serde<'de> for Preview<'a> {}

/// Builder for [Preview].
pub struct PreviewBuilder<'a> {
    base: ObjectBuilder<'a, Null>,
    duration: Option<&'a str>,
    url: Option<Uri<'a>>,
}

impl<'a> PreviewBuilder<'a> {
    pub fn new(preview_type: &'a str, name: &'a str) -> Self {
        PreviewBuilder {
            base: ObjectBuilder::new().object_type(preview_type).name(name),
            duration: None,
            url: None,
        }
    }

    pub fn duration(mut self, dur: &'a str) -> Self {
        self.duration = Some(dur);
        self
    }

    pub fn url(mut self, url: Uri<'a>) -> Self {
        self.url = Some(url);
        self
    }

    pub fn build(self) -> Preview<'a> {
        Preview {
            base: self.base.build(),
            duration: self.duration,
            url: self.url,
        }
    }
}

/// A [Link] is an indirect, qualified reference to a resource identified by a
/// URL. The fundamental model for links is established by
/// [RFC5988](https://www.w3.org/TR/activitystreams-vocabulary/#bib-RFC5988).
/// Many of the properties defined by the Activity Vocabulary allow values that
/// are either instances of [Object] or [Link]. When a [Link] is used, it
/// establishes a qualified relation connecting the subject (the containing
/// object) to the resource identified by the href. Properties of the [Link]
/// are properties of the reference as opposed to properties of the resource.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link<'a> {
    #[serde(rename = "type")]
    pub link_type: &'a str,

    #[serde(flatten, borrow)]
    pub href: Uri<'a>,

    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub rel: Vec<&'a str>, // TODO: RFC5988 validation

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<&'a str>, // TODO: BCP47 language tag

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub preview: Option<Preview<'a>>,
}

impl Link<'_> {
    pub const TYPE: &'static str = "Link";
}

impl<'de: 'a, 'a> Serde<'de> for Link<'a> {}

/// Builder for a [Link] struct.
#[derive(Clone)]
pub struct LinkBuilder<'a> {
    href: UriBuilder<'a>,
    rel: Vec<&'a str>, // TODO: RFC5988 validation
    name: Option<&'a str>,
    hreflang: Option<&'a str>, // TODO: BCP47 language tag
    height: Option<u32>,
    width: Option<u32>,
    preview: Option<Preview<'a>>,
}

impl<'a> LinkBuilder<'a> {
    pub fn new(href: UriBuilder<'a>) -> Self {
        LinkBuilder {
            href,
            rel: Vec::new(),
            name: None,
            hreflang: None,
            height: None,
            width: None,
            preview: None,
        }
    }

    pub fn add_rel(mut self, rel: &'a str) -> Self {
        self.rel.push(rel);
        self
    }

    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub fn hreflang(mut self, hreflang: &'a str) -> Self {
        self.hreflang = Some(hreflang);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn preview(mut self, preview: Preview<'a>) -> Self {
        self.preview = Some(preview);
        self
    }

    pub fn build(self) -> Link<'a> {
        Link {
            link_type: Link::TYPE,
            href: self.href.build(),
            rel: self.rel,
            name: self.name,
            hreflang: self.hreflang,
            height: self.height,
            width: self.width,
            preview: self.preview,
        }
    }
}

/// An [Activity] is a subtype of [Object] that describes some form of action
/// that may happen, is currently happening, or has already happened. The
/// [Activity] type itself serves as an abstract base type for all types of
/// activities. It is important to note that the [Activity] type itself does
/// not carry any specific semantics about the kind of action being taken.
#[derive(Serialize, Deserialize, Debug)]
pub struct Activity<'a> {
    #[serde(flatten)]
    base: Object<'a, Null>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<Actor<'a>>,
    #[serde(skip_serializing_if = "Option::is_none", borrow)]
    pub object: Option<Object<'a, Null>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Object<'a, Null>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<&'a str>, // TODO: Origin
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument: Option<&'a str>, // TODO: Instrument
}

impl<'de: 'a, 'a> Serde<'de> for Activity<'a> {}

impl<'a> std::ops::Deref for Activity<'a> {
    type Target = Object<'a, Null>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for an [Activity].
#[derive(Clone)]
pub struct ActivityBuilder<'a> {
    base: ObjectBuilder<'a, Null>,
    actor: Option<ActorBuilder<'a>>,
    object: Option<ObjectBuilder<'a, Null>>,
    target: Option<ObjectBuilder<'a, Null>>,
    result: Option<&'a str>,
    origin: Option<&'a str>,
    instrument: Option<&'a str>,
}

impl<'a> ActivityBuilder<'a> {
    pub fn new(activity_type: &'a str, summary: &'a str) -> Self {
        ActivityBuilder {
            base: ObjectBuilder::new()
                .object_type(activity_type)
                .summary(summary),
            actor: None,
            object: None,
            target: None,
            result: None,
            origin: None,
            instrument: None,
        }
    }

    pub fn published(&mut self, datetime: DateTime<Utc>) -> Self {
        self.base.published(datetime);
        self.clone()
    }

    pub fn actor(&mut self, actor: ActorBuilder<'a>) -> Self {
        self.actor = Some(actor);
        self.clone()
    }

    pub fn object(&mut self, object: ObjectBuilder<'a, Null>) -> Self {
        self.object = Some(object);
        self.clone()
    }

    pub fn target(&mut self, target: ObjectBuilder<'a, Null>) -> Self {
        self.target = Some(target);
        self.clone()
    }

    pub fn result(&mut self, result: &'a str) -> Self {
        self.result = Some(result);
        self.clone()
    }

    pub fn origin(&mut self, origin: &'a str) -> Self {
        self.origin = Some(origin);
        self.clone()
    }

    pub fn instrument(&mut self, instrument: &'a str) -> Self {
        self.instrument = Some(instrument);
        self.clone()
    }

    pub fn build(self) -> Activity<'a> {
        Activity {
            base: self.base.build(),
            actor: self.actor.map(|a| a.build()),
            object: self.object.map(|o| o.build()),
            target: self.target.map(|t| t.build()),
            result: self.result,
            origin: self.origin,
            instrument: self.instrument,
        }
    }
}

/// Instances of [IntransitiveActivity] are a subtype of [Activity] representing
/// intransitive actions. The object property is therefore inappropriate for
/// these activities.
#[derive(Serialize, Deserialize, Debug)]
pub struct IntransitiveActivity<'a> {
    #[serde(flatten, borrow)]
    base: Activity<'a>,
}

impl<'de: 'a, 'a> Serde<'de> for IntransitiveActivity<'a> {}

impl<'a> std::ops::Deref for IntransitiveActivity<'a> {
    type Target = Activity<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for an [IntransitiveActivity].
#[derive(Clone)]
pub struct IntransitiveActivityBuilder<'a> {
    base: ActivityBuilder<'a>,
}

impl<'a> IntransitiveActivityBuilder<'a> {
    pub fn new(activity_type: &'a str, summary: &'a str) -> Self {
        IntransitiveActivityBuilder {
            base: ActivityBuilder::new(activity_type, summary),
        }
    }

    pub fn published(mut self, datetime: DateTime<Utc>) -> Self {
        self.base.published(datetime);
        self
    }

    pub fn actor(mut self, actor: ActorBuilder<'a>) -> Self {
        self.base.actor(actor);
        self
    }

    pub fn target(mut self, target: ObjectBuilder<'a, Null>) -> Self {
        self.base.target(target);
        self
    }

    pub fn result(mut self, result: &'a str) -> Self {
        self.base.result(result);
        self
    }

    pub fn origin(mut self, origin: &'a str) -> Self {
        self.base.origin(origin);
        self
    }

    pub fn instrument(mut self, instrument: &'a str) -> Self {
        self.base.instrument(instrument);
        self
    }

    pub fn build(self) -> IntransitiveActivity<'a> {
        IntransitiveActivity {
            base: self.base.build(),
        }
    }
}

/// A [Collection] is a subtype of [Object] that represents ordered or unordered
/// sets of [Object] or [Link] instances. Refer to the Activity Streams 2.0 Core
/// specification for a complete description of the [Collection] type.
#[derive(Serialize, Deserialize, Debug)]
pub struct Collection<'a, CollectionT> {
    #[serde(flatten, borrow)]
    base: Object<'a, Null>,

    #[serde(rename = "totalItems", skip_serializing_if = "Option::is_none")]
    pub total_items: Option<usize>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<CollectionT>,
}

impl<'de: 'a, 'a, CollectionT> Serde<'de> for Collection<'de, CollectionT> where
    CollectionT: Serde<'de>
{
}

impl<'a, CollectionT> std::ops::Deref for Collection<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    type Target = Object<'a, Null>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for a [Collection].
pub struct CollectionBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    base: ObjectBuilder<'a, Null>,
    items: Vec<CollectionT>,
}

impl<'a, CollectionT> CollectionBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    pub fn new(collection_type: &'a str, items: Vec<CollectionT>) -> Self {
        CollectionBuilder {
            base: ObjectBuilder::new().object_type(collection_type),
            items,
        }
    }

    pub fn build(self) -> Collection<'a, CollectionT> {
        Collection {
            base: self.base.build(),
            total_items: match self.items.is_empty() {
                true => None,
                false => Some(self.items.len()),
            },
            items: self.items,
        }
    }
}

/// A subtype of [Collection] in which members of the logical collection are
/// assumed to always be strictly ordered.
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderedCollection<'a, CollectionT> {
    #[serde(flatten, borrow)]
    base: Object<'a, Null>,

    #[serde(rename = "totalItems", skip_serializing_if = "Option::is_none")]
    pub total_items: Option<usize>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(rename = "orderedItems")]
    pub ordered_items: Vec<CollectionT>,
}

impl<'de: 'a, 'a, CollectionT> Serde<'de> for OrderedCollection<'de, CollectionT> where
    CollectionT: Serde<'de>
{
}

impl<'a, CollectionT> std::ops::Deref for OrderedCollection<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    type Target = Object<'a, Null>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for an [OrderedCollection].
pub struct OrderedCollectionBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    base: ObjectBuilder<'a, Null>,
    ordered_items: Vec<CollectionT>,
}

impl<'a, CollectionT> OrderedCollectionBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    pub fn new(collection_type: &'a str, ordered_items: Vec<CollectionT>) -> Self {
        OrderedCollectionBuilder {
            base: ObjectBuilder::new().object_type(collection_type),
            ordered_items,
        }
    }

    pub fn build(self) -> OrderedCollection<'a, CollectionT> {
        OrderedCollection {
            base: self.base.build(),
            total_items: match self.ordered_items.is_empty() {
                true => None,
                false => Some(self.ordered_items.len()),
            },
            ordered_items: self.ordered_items,
        }
    }
}

/// Used to represent distinct subsets of items from a [Collection]. Refer to
/// the Activity Streams 2.0 Core for a complete description of the
/// [CollectionPage] object.
#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionPage<'a, CollectionT> {
    #[serde(flatten, borrow)]
    base: Collection<'a, CollectionT>,

    #[serde(rename = "partOf")]
    pub part_of: String,

    pub next: Option<String>,

    pub prev: Option<String>,
}

impl<'de: 'a, 'a, CollectionT> Serde<'de> for CollectionPage<'de, CollectionT> where
    CollectionT: Serde<'de>
{
}

impl<'a, CollectionT> std::ops::Deref for CollectionPage<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    type Target = Collection<'a, CollectionT>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for a [CollectionPage].
pub struct CollectionPageBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    base: CollectionBuilder<'a, CollectionT>,
    part_of: &'a http::Uri,
    next: Option<&'a http::Uri>,
    prev: Option<&'a http::Uri>,
}

impl<'a, CollectionT> CollectionPageBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    pub fn new(collection_type: &'a str, items: Vec<CollectionT>, part_of: &'a http::Uri) -> Self {
        CollectionPageBuilder {
            base: CollectionBuilder::new(collection_type, items),
            part_of,
            next: None,
            prev: None,
        }
    }

    pub fn next(mut self, next: &'a http::Uri) -> Self {
        self.next = Some(next);
        self
    }

    pub fn prev(mut self, prev: &'a http::Uri) -> Self {
        self.prev = Some(prev);
        self
    }

    pub fn build(self) -> CollectionPage<'a, CollectionT> {
        CollectionPage {
            base: self.base.build(),
            part_of: self.part_of.to_string(),
            next: self.next.map(|n| n.to_string()),
            prev: self.prev.map(|p| p.to_string()),
        }
    }
}

/// Used to represent ordered subsets of items from an [OrderedCollection].
/// Refer to the Activity Streams 2.0 Core for a complete description of
/// the [OrderedCollectionPage] object.
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderedCollectionPage<'a, CollectionT> {
    #[serde(flatten, borrow)]
    base: OrderedCollection<'a, CollectionT>,

    #[serde(rename = "partOf")]
    pub part_of: String,

    pub next: Option<String>,

    pub prev: Option<String>,
}

impl<'de: 'a, 'a, CollectionT> Serde<'de> for OrderedCollectionPage<'de, CollectionT> where
    CollectionT: Serde<'de>
{
}

impl<'a, CollectionT> std::ops::Deref for OrderedCollectionPage<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    type Target = OrderedCollection<'a, CollectionT>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// Builder for a [OrderedCollectionPage].
pub struct OrderedCollectionPageBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    base: OrderedCollectionBuilder<'a, CollectionT>,
    part_of: &'a http::Uri,
    next: Option<&'a http::Uri>,
    prev: Option<&'a http::Uri>,
}

impl<'a, CollectionT> OrderedCollectionPageBuilder<'a, CollectionT>
where
    CollectionT: Serde<'a>,
{
    pub fn new(collection_type: &'a str, items: Vec<CollectionT>, part_of: &'a http::Uri) -> Self {
        OrderedCollectionPageBuilder {
            base: OrderedCollectionBuilder::new(collection_type, items),
            part_of,
            next: None,
            prev: None,
        }
    }

    pub fn next(mut self, next: &'a http::Uri) -> Self {
        self.next = Some(next);
        self
    }

    pub fn prev(mut self, prev: &'a http::Uri) -> Self {
        self.prev = Some(prev);
        self
    }

    pub fn build(self) -> OrderedCollectionPage<'a, CollectionT> {
        OrderedCollectionPage {
            base: self.base.build(),
            part_of: self.part_of.to_string(),
            next: self.next.map(|n| n.to_string()),
            prev: self.prev.map(|p| p.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::Result;

    #[test]
    fn serialize_object() {
        let object: Object<Null> = ObjectBuilder::new().name("name").build();
        let actual = Document::new(ContextBuilder::new().language("en").build(), object);
        let expected = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams",
    "@language": "en"
  },
  "name": "name"
}"#;
        assert!(actual.to_json_pretty().is_ok());
        assert_eq!(actual.to_json_pretty().unwrap(), expected)
    }

    #[test]
    fn deserialize_object() {
        let actual = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams",
    "@language": "en"
  },
  "name": "name"
}"#;
        let document: Document<Object<Null>> = Document::from_json(&actual).unwrap();
        assert_eq!(document.context.language, Some("en"));
        let object = document.object as Object<Null>;
        assert_eq!(object.name, Some("name"));
    }

    #[test]
    fn deserialize_object_malformed() {
        let actual = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams",
    "@language": "en"
  },
}"#;
        let result: Result<Document<Object<Null>>> = Document::from_json(&actual);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_link() {
        let actual = Document::new(
            ContextBuilder::new().build(),
            LinkBuilder::new(UriBuilder::new(
                "http://example.org/abc".parse::<http::Uri>().unwrap(),
            ))
            .name("An example link")
            .hreflang("en")
            .build(),
        );
        let expected = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Link",
  "href": "http://example.org/abc",
  "name": "An example link",
  "hreflang": "en"
}"#;
        assert!(actual.to_json_pretty().is_ok());
        assert_eq!(actual.to_json_pretty().unwrap(), expected);
    }

    #[test]
    fn deserialize_link() {
        let actual = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Link",
  "href": "http://example.org/abc",
  "name": "An example link",
  "hreflang": "en"
}"#;
        let document: Document<Link> = Document::from_json(&actual).unwrap();
        let link = document.object as Link;
        assert_eq!(link.link_type, "Link");
        assert_eq!(link.href.href, "http://example.org/abc");
        assert_eq!(link.name, Some("An example link"));
        assert_eq!(link.hreflang, Some("en"));
    }

    #[test]
    fn serialize_preview() {
        let actual = Document::new(
            ContextBuilder::new().build(),
            PreviewBuilder::new("Video", "Trailer")
                .duration("PT1M")
                .url(
                    UriBuilder::new(
                        "http://example.org/trailer.mkv"
                            .parse::<http::Uri>()
                            .unwrap(),
                    )
                    .media_type("video/mkv")
                    .build(),
                )
                .build(),
        );
        let expected = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Video",
  "name": "Trailer",
  "duration": "PT1M",
  "url": {
    "href": "http://example.org/trailer.mkv",
    "mediaType": "video/mkv"
  }
}"#;
        assert!(actual.to_json_pretty().is_ok());
        assert_eq!(actual.to_json_pretty().unwrap(), expected);
    }

    #[test]
    fn deserialize_preview() {
        let actual = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Video",
  "name": "Trailer",
  "duration": "PT1M",
  "url": {
    "href": "http://example.org/trailer.mkv",
    "mediaType": "video/mkv"
  }
}"#;
        let document: Document<Preview> = Document::from_json(&actual).unwrap();
        let preview = document.object as Preview;
        assert_eq!(preview.base.object_type, Some("Video"));
        assert_eq!(preview.base.name, Some("Trailer"));
        assert_eq!(preview.duration, Some("PT1M"));
        assert!(preview.url.is_some());
        assert_eq!(
            preview.url.as_ref().unwrap().href,
            "http://example.org/trailer.mkv".to_string()
        );
        assert_eq!(preview.url.as_ref().unwrap().media_type, Some("video/mkv"));
    }

    #[test]
    fn serialize_activity() {
        let actual = Document::new(
            ContextBuilder::new().build(),
            ActivityBuilder::new("Activity", "Sally did something to a note")
                .actor(ActorBuilder::new("Person").name("Sally"))
                .object(ObjectBuilder::new().object_type("Note").name("A Note"))
                .build(),
        );

        let expected = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Activity",
  "summary": "Sally did something to a note",
  "actor": {
    "type": "Person",
    "name": "Sally"
  },
  "object": {
    "type": "Note",
    "name": "A Note"
  }
}"#;
        assert!(actual.to_json_pretty().is_ok());
        assert_eq!(actual.to_json_pretty().unwrap(), expected);
    }

    #[test]
    fn deserialize_activity() {
        let actual = r#"{
  "@context": {
    "@vocab": "https://www.w3.org/ns/activitystreams"
  },
  "type": "Activity",
  "summary": "Sally did something to a note",
  "actor": {
    "type": "Person",
    "name": "Sally"
  },
  "object": {
    "type": "Note",
    "name": "A Note"
  }
}"#;
        let document: Document<Activity> = Document::from_json(actual).unwrap();
        let activity = document.object as Activity;
        assert_eq!(activity.object_type, Some("Activity"));
        assert_eq!(activity.summary, Some("Sally did something to a note"));

        assert!(activity.actor.is_some());
        let actor = activity.actor.as_ref().unwrap();
        assert_eq!(actor.object_type, Some("Person"));
        assert_eq!(actor.name, Some("Sally"));

        assert!(activity.object.is_some());
        let object = activity.object.as_ref().unwrap();
        assert_eq!(object.object_type, Some("Note"));
        assert_eq!(object.name, Some("A Note"));
    }
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::Content;

/// The request body contains data with the following structure
#[derive(Serialize, Deserialize, Default)]
pub struct GeminiRequestBody {
    /// Required. The content of the current conversation with the model.
    /// For single-turn queries, this is a single instance.
    /// For multi-turn queries like chat, this is a repeated field that contains the conversation history and the latest request.
    pub contents: Vec<Content>,
    /// Optional. A list of Tools the Model may use to generate the next response.
    /// A Tool is a piece of code that enables the system to interact with external systems to perform an action,
    /// or set of actions, outside of knowledge and scope of the Model.
    /// Supported Tools are Function and codeExecution. Refer to the Function calling and the Code execution guides to learn more.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Optional. Configuration options for model generation and outputs.
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
}

/// Configuration options for model generation and outputs. Not all parameters are configurable for every model.
#[derive(Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Optional. The set of character sequences (up to 5) that will stop output generation.
    /// If specified, the API will stop at the first appearance of a stop_sequence.
    /// The stop sequence will not be included as part of the response.
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopSequences")]
    pub stop_sequences: Option<Vec<String>>,
    /// Optional. MIME type of the generated candidate text. Supported MIME types are: text/plain: (default) Text output.
    /// application/json: JSON response in the response candidates. Refer to the docs for a list of all supported text MIME types.
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseMimeType")]
    pub response_mime_type: Option<String>,
    /// Optional. Output schema of the generated candidate text. Schemas must be a subset of the OpenAPI schema and can be objects, primitives or arrays.
    /// If set, a compatible responseMimeType must also be set. Compatible MIME types: application/json: Schema for JSON response.
    /// Refer to the JSON text generation guide for more details.
    #[serde(skip_serializing_if = "Option::is_none", rename = "responseSchema")]
    pub response_schema: Option<Schema>,
    /// Optional. Number of generated responses to return.
    /// Currently, this value can only be set to 1. If unset, this will default to 1.
    #[serde(skip_serializing_if = "Option::is_none", rename = "candidateCount")]
    pub candidate_count: Option<isize>,
    /// Optional. The maximum number of tokens to include in a response candidate.
    /// Note: The default value varies by model, see the Model.output_token_limit attribute of the Model returned from the getModel function.
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxOutputTokens")]
    pub max_output_tokens: Option<isize>,
    /// Optional. Controls the randomness of the output.
    /// Note: The default value varies by model, see the Model.temperature attribute of the Model returned from the getModel function.
    /// Values can range from [0.0, 2.0].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional. The maximum cumulative probability of tokens to consider when sampling.
    /// The model uses combined Top-k and Top-p (nucleus) sampling.
    /// Tokens are sorted based on their assigned probabilities so that only the most likely tokens are considered.
    /// Top-k sampling directly limits the maximum number of tokens to consider,
    /// while Nucleus sampling limits the number of tokens based on the cumulative probability.
    /// Note: The default value varies by Model and is specified by theModel.top_p attribute returned from the getModel function.
    /// An empty topK attribute indicates that the model doesn't apply top-k sampling and doesn't allow setting topK on requests.
    #[serde(skip_serializing_if = "Option::is_none", rename = "topP")]
    pub top_p: Option<f64>,
    /// Optional. The maximum number of tokens to consider when sampling.
    /// Gemini models use Top-p (nucleus) sampling or a combination of Top-k and nucleus sampling. Top-k sampling considers the set of topK most probable tokens.
    /// Models running with nucleus sampling don't allow topK setting.
    /// Note: The default value varies by Model and is specified by theModel.top_p attribute returned from the getModel function.
    /// An empty topK attribute indicates that the model doesn't apply top-k sampling and doesn't allow setting topK on requests.
    #[serde(skip_serializing_if = "Option::is_none", rename = "topK")]
    pub top_k: Option<isize>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: Some(1.0),
            top_k: Some(64),
            top_p: Some(0.95),
            max_output_tokens: Some(8192),
            response_mime_type: Some("text/plain".to_owned()),
            stop_sequences: None,
            response_schema: None,
            candidate_count: None,
        }
    }
}

/// Tool details that the model may use to generate response.
/// A Tool is a piece of code that enables the system to interact with external systems to perform an action, or set of actions,
/// outside of knowledge and scope of the model.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Optional. A list of FunctionDeclarations available to the model that can be used for function calling.
    /// The model or system does not execute the function.
    /// Instead the defined function may be returned as a [FunctionCall][content.part.function_call] with arguments to the client side for execution. The model may decide to call a subset of these functions by populating [FunctionCall][content.part.function_call] in the response. The next conversation turn may contain a [FunctionResponse][content.part.function_response] with the [content.role] "function" generation context for the next model turn.
    #[serde(skip_serializing_if = "Option::is_none", rename = "functionDeclarations")]
    pub function_declarations: Option<Vec<FunctionDeclaration>>,
    /// Optional. Enables the model to execute code as part of generation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "codeExecution")]
    pub code_execution: Option<CodeExecution>,
}

/// Structured representation of a function declaration as defined by the OpenAPI 3.03 specification.
/// Included in this declaration are the function name and parameters.
/// This FunctionDeclaration is a representation of a block of code that can be used as a Tool by the model and executed by the client.
#[derive(Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    /// Required. The name of the function. Must be a-z, A-Z, 0-9, or contain underscores and dashes, with a maximum length of 63.
    pub name: String,
    /// Required. A brief description of the function.
    pub description: String,
    /// Optional. Describes the parameters to this function.
    /// Reflects the Open API 3.03 Parameter Object string Key: the name of the parameter.
    /// Parameter names are case sensitive. Schema Value: the Schema defining the type used for the parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
}

/// The Schema object allows the definition of input and output data types.
/// These types can be objects, but also primitives and arrays.
/// Represents a select subset of an OpenAPI 3.0 schema object.
#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Required. Data type.
    #[serde(rename = "type")]
    pub type0: Type,
    /// Optional. The format of the data. This is used only for primitive datatypes.
    /// Supported formats: for NUMBER type: float, double for INTEGER type: int32, int64 for STRING type: enum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Optional. A brief description of the parameter. This could contain examples of use. Parameter description may be formatted as Markdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional. Indicates if the value may be null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    /// Optional. Possible values of the element of Type.STRING with enum format.
    /// For example we can define an Enum Direction as : {type:STRING, format:enum, enum:["EAST", NORTH", "SOUTH", "WEST"]}
    #[serde(skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum0: Option<Vec<String>>,
    /// Optional. Maximum number of the elements for Type.ARRAY.
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxItems")]
    pub max_items: Option<String>,
    /// Optional. Properties of Type.OBJECT.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Box<Schema>>>,
    /// Optional. Required properties of Type.OBJECT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Optional. Schema of the elements of Type.ARRAY.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
}

/// Type contains the list of OpenAPI data types as defined by https://spec.openapis.org/oas/v3.0.3#data-types
#[derive(Clone, Serialize, Deserialize)]
pub enum Type {
    /// Not specified, should not be used.
    #[serde(rename = "TYPE_UNSPECIFIED")]
    TypeUnspecified,
    /// String type.
    #[serde(rename = "STRING")]
    String,
    /// Number type.
    #[serde(rename = "NUMBER")]
    Number,
    /// Integer type.
    #[serde(rename = "INTEGER")]
    Integer,
    /// Boolean type.
    #[serde(rename = "BOOLEAN")]
    Boolean,
    /// Array type.
    #[serde(rename = "ARRAY")]
    Array,
    /// Object type.
    #[serde(rename = "OBJECT")]
    Object,
}

/// This type has no fields.
/// Tool that executes code generated by the model, and automatically returns the result to the model.
/// See also ExecutableCode and CodeExecutionResult which are only generated when using this tool.
#[derive(Clone, Serialize, Deserialize)]
pub struct CodeExecution;

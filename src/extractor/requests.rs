use super::*;

pub(crate) fn extract_requests_from_openapi(openapi: &OpenAPI) -> Vec<Request> {
    let mut requests = Vec::new();

    // IndexMap<String, ReferenceOr<PathItem>>
    for (path, data) in openapi.paths.clone() {
        // https://docs.rs/openapiv3/0.3.0/openapiv3/struct.PathItem.html

        for request in extract_request_from_openapi(path, data) {
            requests.push(request);
        }
    }

    requests
}

fn extract_request_from_openapi(path: String, data: ReferenceOr<PathItem>) -> Vec<Request> {
    let mut requests = Vec::new();

    if let ReferenceOr::Item(data) = data {
        data.get.map(|operation| {
            requests.push(openapi_operation_to_request(
                path.clone(),
                Method::Get,
                operation,
            ))
        });
        data.put.map(|operation| {
            requests.push(openapi_operation_to_request(
                path.clone(),
                Method::Put,
                operation,
            ))
        });
        data.post.map(|operation| {
            requests.push(openapi_operation_to_request(
                path.clone(),
                Method::Post,
                operation,
            ))
        });
    };

    requests
}

fn openapi_operation_to_request(path: String, method: Method, operation: Operation) -> Request {
    // https://docs.rs/openapiv3/0.3.0/openapiv3/struct.Operation.html

    Request {
        name: operation.operation_id.unwrap_or("".to_string()),
        path,
        params: extract_params_from_openapi(operation.parameters)
            .into_iter()
            .map(|v| Box::new(v))
            .collect(),
        method,
        response_type: extract_response_type_from_openapi(operation.responses),
        error_type: None,
    }
}

fn extract_response_type_from_openapi(responses: Responses) -> Option<Box<VariableType>> {
    if let Some(response) = responses.default {
        if let ReferenceOr::Item(response) = response {
            for (name, content) in response.content {
                if let Some(reference) = content.schema {
                    if let ReferenceOr::Reference{ reference } = reference {
                        if let Some(complex_type) = reference.split("/").last() {
                            return Some(Box::new(
                                VariableType::ComplexType(complex_type.into())
                            ));
                        } else {
                            return variable_type_from_string("String"); // FIX
                        }
                    }
                }
            }
        }
    };
    None
}

fn variable_type_from_string(t: &str) -> Option<Box<VariableType>> {
    None
}

fn extract_params_from_openapi(parameters: Vec<ReferenceOr<Parameter>>) -> Vec<Variable> {
    let mut variables = Vec::new();

    for parameter in parameters {
        if let ReferenceOr::Item(parameter) = parameter {
            match parameter {
                Parameter::Query { parameter_data, .. } => variables.push(Variable {
                    name: parameter_data.name,
                    optional: parameter_data.required,
                    value: None,
                    variable_type: VariableType::StringType,
                }),
                Parameter::Path { parameter_data, .. } => variables.push(Variable {
                    name: parameter_data.name,
                    optional: parameter_data.required,
                    value: None,
                    variable_type: VariableType::StringType,
                }),
                _ => (),
            }
        }
    }

    variables
}

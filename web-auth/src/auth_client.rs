use oauth2::{
    basic::{
        BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
        BasicTokenResponse,
    },
    Client, EndpointNotSet, EndpointSet, StandardRevocableToken,
};

pub type SpecialClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointSet,
> = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
>;

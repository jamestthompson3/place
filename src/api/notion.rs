use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct PublicPageData {
    spaceName: Option<String>,
    spaceId: Option<String>,
    spaceDomain: Option<String>,
    canJoinSpace: Option<bool>,
    userHasExplicitAccess: Option<bool>,
    hasPublicAccess: Option<bool>,
    ownerUserId: Option<String>,
    betaEnabled: Option<bool>,
}

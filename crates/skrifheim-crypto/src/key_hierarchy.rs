use crate::KeyScope;

pub(crate) fn is_valid_parent(child: KeyScope, parent: KeyScope) -> bool {
    match (child, parent) {
        (KeyScope::Deployment { .. }, KeyScope::RootTrust) => true,
        (
            KeyScope::Region { deployment_id, .. },
            KeyScope::Deployment {
                deployment_id: parent_deployment,
            },
        ) => deployment_id == parent_deployment,
        (
            KeyScope::Tenant {
                deployment_id,
                region_id,
                ..
            },
            KeyScope::Region {
                deployment_id: parent_deployment,
                region_id: parent_region,
            },
        ) => deployment_id == parent_deployment && region_id == parent_region,
        (
            KeyScope::Compartment { tenant_id, .. },
            KeyScope::Tenant {
                tenant_id: parent_tenant,
                ..
            },
        ) => tenant_id == parent_tenant,
        (
            KeyScope::Segment {
                tenant_id,
                compartment_id,
                ..
            }
            | KeyScope::Data {
                tenant_id,
                compartment_id,
                ..
            },
            KeyScope::Compartment {
                tenant_id: parent_tenant,
                compartment_id: parent_compartment,
            },
        ) => tenant_id == parent_tenant && compartment_id == parent_compartment,
        _ => false,
    }
}

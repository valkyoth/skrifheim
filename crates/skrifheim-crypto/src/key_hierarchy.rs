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
            KeyScope::Compartment {
                deployment_id,
                region_id,
                tenant_id,
                ..
            },
            KeyScope::Tenant {
                deployment_id: parent_deployment,
                region_id: parent_region,
                tenant_id: parent_tenant,
            },
        ) => {
            deployment_id == parent_deployment
                && region_id == parent_region
                && tenant_id == parent_tenant
        }
        (
            KeyScope::Segment {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
                ..
            }
            | KeyScope::Data {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
                ..
            },
            KeyScope::Compartment {
                deployment_id: parent_deployment,
                region_id: parent_region,
                tenant_id: parent_tenant,
                compartment_id: parent_compartment,
            },
        ) => {
            deployment_id == parent_deployment
                && region_id == parent_region
                && tenant_id == parent_tenant
                && compartment_id == parent_compartment
        }
        _ => false,
    }
}

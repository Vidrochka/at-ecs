#[cfg(feature = "anthill-di")]
#[tokio::test]
async fn registration_id_dublicate_err() {
    use anthill_di::transient;
    use type_uuid::TypeUuid;

    use crate::{system::{
        ioc::system_from_ioc_context, system::ISystem, systems_configuration_flow_manager::{
            SystemRegistrationError, SystemsFlowConfigurationManager
        }
    }, world::World};

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000001"]
    struct SystemA {}

    #[async_trait::async_trait]
    impl ISystem for SystemA {
        async fn tick(&mut self, _: &World) {
            println!("{:?}", self)
        }
    }

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000001"]
    struct SystemB {}

    #[async_trait::async_trait]
    impl ISystem for SystemB {
        async fn tick(&mut self, _: &World) {
            println!("{:?}", self)
        }
    }

    let ioc = anthill_di::DependencyContext::new_root();

    ioc.register(transient(|_: ()| async {
        Ok(SystemA {})
    })).await;

    ioc.register(transient(|_: ()| async {
        Ok(SystemB {})
    })).await;

    let mut sm = SystemsFlowConfigurationManager::default();

    sm.register(system_from_ioc_context::<SystemA>(ioc.clone())).unwrap();
    assert_eq!(
        sm.register(system_from_ioc_context::<SystemB>(ioc)),
        Err(SystemRegistrationError::SystemAlreadyExists(
            sm.system_node_typed::<SystemA>()
                .unwrap()
                .configuration()
                .system_id()
        ))
    );
}
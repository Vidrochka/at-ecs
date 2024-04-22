#[cfg(feature = "anthill-di")]
#[tokio::test]
async fn registration_single_ok() {
    use anthill_di::transient;
    use type_uuid::TypeUuid;

    use crate::{system::{
        ioc::system_from_ioc_context, system::ISystem, systems_configuration_flow_manager::SystemsFlowConfigurationManager
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

    let ioc = anthill_di::DependencyContext::new_root();

    ioc.register(transient(|_: ()| async {
        Ok(SystemA {})
    })).await;

    let mut sm = SystemsFlowConfigurationManager::default();

    sm.register(system_from_ioc_context::<SystemA>(ioc)).unwrap();
}
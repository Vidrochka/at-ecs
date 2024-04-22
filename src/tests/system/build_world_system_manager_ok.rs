#[cfg(feature = "anthill-di")]
#[tokio::test]
async fn build_world_system_manager_ok() {
    use anthill_di::transient;
    use type_uuid::TypeUuid;

    use crate::{system::{
        ioc::system_from_ioc_context, system::ISystem, systems_configuration_flow_manager::SystemsFlowConfigurationManager
    }, world::World};

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000001"]
    struct SystemA {}

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000002"]
    struct SystemB {}

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000003"]
    struct SystemC {}

    #[derive(Debug, TypeUuid)]
    #[uuid = "00000000-0000-0000-0000-000000000004"]
    struct SystemD {}

    #[async_trait::async_trait]
    impl ISystem for SystemA {
        async fn tick(&mut self, _: &World) {
            println!("{:?}", self)
        }
    }

    #[async_trait::async_trait]
    impl ISystem for SystemB {
        async fn tick(&mut self, _: &World) {
            println!("{:?}", self)
        }
    }

    #[async_trait::async_trait]
    impl ISystem for SystemC {
        async fn tick(&mut self, _: &World) {
            println!("{:?}", self)
        }
    }

    #[async_trait::async_trait]
    impl ISystem for SystemD {
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

    ioc.register(transient(|_: ()| async {
        Ok(SystemC {})
    })).await;

    ioc.register(transient(|_: ()| async {
        Ok(SystemD {})
    })).await;

    let mut sm = SystemsFlowConfigurationManager::default();

    let system_a_id = sm.register(system_from_ioc_context::<SystemA>(ioc.clone())).unwrap();
    let system_c_id = sm.register(system_from_ioc_context::<SystemC>(ioc.clone())).unwrap();
    let system_b_id = sm.register(system_from_ioc_context::<SystemB>(ioc.clone())).unwrap();
    let system_d_id = sm.register(system_from_ioc_context::<SystemD>(ioc.clone())).unwrap();

    sm.add_link(&system_a_id, &system_c_id).unwrap();
    sm.add_link(&system_a_id, &system_b_id).unwrap();
    sm.add_link(&system_c_id, &system_d_id).unwrap();
    sm.add_link(&system_b_id, &system_d_id).unwrap();

    let res = sm.build_world_systems_manager([system_d_id, system_a_id, system_b_id, system_c_id].into()).await;

    println!("{res:#?}");
}
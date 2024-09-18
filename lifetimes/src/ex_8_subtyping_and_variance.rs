/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! This example shows how to use subtyping and variance in Rust using more sophisticated
//! examples that use traits, supertraits, and generics. No lifetimes are used here.

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    /* Simple concrete types. */

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Dimens2D {
        pub width: usize,
        pub height: usize,
        pub x: usize,
        pub y: usize,
    }

    /* Traits and their subtyping relationships. */

    pub trait View<T: Copy + Debug> {
        fn get_dimens(&self) -> T;
        fn set_dimens(&mut self, dimens: T);
    }

    pub trait Component<T: Copy + Debug>: View<T> {
        fn get_as_view(&self) -> &dyn View<T>;
        fn get_as_view_mut(&mut self) -> &mut dyn View<T>;
        fn render(&self);
    }

    /* Trait implementations. */

    /// Concrete type [Button2D] that implements the [View] & [Component] traits.
    pub mod button {
        use super::*;

        #[derive(Clone, Copy, Debug)]
        pub struct Button2D {
            pub dimens: Dimens2D,
        }

        impl View<Dimens2D> for Button2D {
            fn get_dimens(&self) -> Dimens2D {
                self.dimens
            }

            fn set_dimens(&mut self, dimens: Dimens2D) {
                self.dimens = dimens;
            }
        }

        impl Component<Dimens2D> for Button2D {
            fn get_as_view(&self) -> &dyn View<Dimens2D> {
                self
            }

            fn get_as_view_mut(&mut self) -> &mut dyn View<Dimens2D> {
                self
            }

            fn render(&self) {
                println!("Button2D: {:?}", self.dimens);
            }
        }
    }

    /// Concrete type [Label2D] that implements the [View] & [Component] traits.
    pub mod label {
        use super::*;

        #[derive(Clone, Copy, Debug)]
        pub struct Label2D {
            pub dimens: Dimens2D,
        }

        impl View<Dimens2D> for Label2D {
            fn get_dimens(&self) -> Dimens2D {
                self.dimens
            }

            fn set_dimens(&mut self, dimens: Dimens2D) {
                self.dimens = dimens;
            }
        }

        impl Component<Dimens2D> for Label2D {
            fn get_as_view(&self) -> &dyn View<Dimens2D> {
                self
            }

            fn get_as_view_mut(&mut self) -> &mut dyn View<Dimens2D> {
                self
            }

            fn render(&self) {
                println!("Label2D: {:?}", self.dimens);
            }
        }
    }

    /* Functions that work with the subtrait. */

    fn component_render<T: Copy + Debug>(component: &dyn Component<T>) {
        component.render();
    }

    fn component_get_size<T: Copy + Debug>(component: &dyn Component<T>) -> T {
        component.get_as_view().get_dimens()
    }

    fn component_set_size<T: Copy + Debug>(component: &mut dyn Component<T>, dimens: T) {
        component.get_as_view_mut().set_dimens(dimens);
    }

    /* Functions that work with the supertrait. */

    fn view_get_size<T: Copy + Debug>(view: &dyn View<T>) -> T {
        view.get_dimens()
    }

    fn view_set_size<T: Copy + Debug>(view: &mut dyn View<T>, dimens: T) {
        view.set_dimens(dimens);
    }

    /* Tests. */

    const SMALL: Dimens2D = Dimens2D {
        width: 100,
        height: 50,
        x: 10,
        y: 20,
    };

    const LARGE: Dimens2D = Dimens2D {
        width: 200,
        height: 100,
        x: 30,
        y: 40,
    };

    #[test]
    fn test_access_as_component() {
        let mut button_component = button::Button2D { dimens: SMALL };
        let mut label_component = label::Label2D { dimens: SMALL };

        component_render(&button_component);
        component_render(&label_component);

        let button_size = component_get_size(&button_component);
        let label_size = component_get_size(&label_component);

        assert_eq!(button_size, button_component.dimens);
        assert_eq!(label_size, label_component.dimens);

        component_set_size(&mut button_component, LARGE);
        component_set_size(&mut label_component, LARGE);

        assert_eq!(button_component.dimens, LARGE,);
        assert_eq!(label_component.dimens, LARGE,);
    }

    #[test]
    fn test_access_component_as_view() {
        let mut button_component = button::Button2D { dimens: SMALL };
        let mut label_component = label::Label2D { dimens: SMALL };

        let button_size = button_component.get_as_view().get_dimens();
        let label_size = button_component.get_as_view().get_dimens();

        assert_eq!(button_size, button_component.dimens);
        assert_eq!(label_size, label_component.dimens);

        button_component.get_as_view_mut().set_dimens(LARGE);
        label_component.get_as_view_mut().set_dimens(LARGE);

        assert_eq!(button_component.dimens, LARGE);
        assert_eq!(label_component.dimens, LARGE);
    }

    #[test]
    fn test_access_as_view() {
        let mut button_component = button::Button2D { dimens: SMALL };
        let mut label_component = label::Label2D { dimens: SMALL };

        let button_size = view_get_size(&button_component);
        let label_size = view_get_size(&label_component);

        assert_eq!(button_size, button_component.dimens);
        assert_eq!(label_size, label_component.dimens);

        view_set_size(&mut button_component, LARGE);
        view_set_size(&mut label_component, LARGE);

        assert_eq!(button_component.dimens, LARGE);
        assert_eq!(label_component.dimens, LARGE);
    }

    #[test]
    fn test_accumulating_components_in_collection() {
        let mut components: Vec<&mut dyn Component<Dimens2D>> = vec![];

        let mut button_component = button::Button2D { dimens: SMALL };
        let mut label_component = label::Label2D { dimens: SMALL };

        components.push(&mut button_component);
        components.push(&mut label_component);

        assert_eq!(components[0].get_dimens(), SMALL);
        assert_eq!(components[1].get_dimens(), SMALL);

        // Access as a component.
        for component in components.iter() {
            component.render();
        }

        // Access as a view.
        for component in components.iter() {
            let size = component.get_dimens();
            println!("Size: {:?}", size);
        }

        // Mutate as view.
        for component in components.iter_mut() {
            component.set_dimens(LARGE);
        }

        assert_eq!(components[0].get_dimens(), LARGE);
        assert_eq!(components[1].get_dimens(), LARGE);
    }
}

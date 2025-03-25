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

    //
    // Concrete types (simple).
    //

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Width(pub usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Height(pub usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct X(pub usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Y(pub usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Dimens2D {
        pub width: Width,
        pub height: Height,
        pub x: X,
        pub y: Y,
    }

    pub struct Dimens2DAlt {
        pub width: usize,
        pub height: usize,
        pub x: usize,
        pub y: usize,
    }

    //
    // Traits.
    //

    /// Super trait.
    pub trait View<InnerStorage: Debug + Copy + Clone + PartialEq + Eq> {
        fn get_dimens(&self) -> InnerStorage;
        fn set_dimens(&mut self, dimens: InnerStorage);
    }

    /// Sub trait.
    pub trait Component<
        InnerStorage: Debug + Copy + Clone + PartialEq + Eq,
        RenderOutput: Debug + Clone,
    >: View<InnerStorage>
    {
        fn render(&self) -> RenderOutput;
    }

    /// Redundant.
    pub trait ComponentAlt<T: Debug + Copy + Clone + PartialEq + Eq, O: Debug + Clone>:
        View<T>
    {
        fn render(&self) -> O;
        fn as_view(&self) -> &dyn View<T>;
        fn as_view_mut(&mut self) -> &mut dyn View<T>;
    }

    //
    // Trait impl.
    //

    pub mod button {
        use super::*;

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

        impl Component<Dimens2D, String> for Button2D {
            fn render(&self) -> String {
                format!(
                    "Button2D: width: {}, height: {}, x: {}, y: {}",
                    self.dimens.width.0, self.dimens.height.0, self.dimens.x.0, self.dimens.y.0
                )
            }
        }
    }

    pub mod label {
        use super::*;

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

        impl Component<Dimens2D, String> for Label2D {
            fn render(&self) -> String {
                format!(
                    "Label2D: width: {}, height: {}, x: {}, y: {}",
                    self.dimens.width.0, self.dimens.height.0, self.dimens.x.0, self.dimens.y.0
                )
            }
        }
    }

    //
    // fns that work with the sub trait. (Component)
    //

    pub mod subtrait_fns {
        use super::*;

        pub fn comp_render<T: Debug + Copy + Clone + PartialEq + Eq, O: Debug + Clone>(
            arg: &dyn Component<T, O>,
        ) -> O {
            arg.render()
        }

        pub fn comp_get_size<T: Debug + Copy + Clone + PartialEq + Eq, O: Debug + Clone>(
            arg: impl Component<T, O>,
        ) -> T {
            arg.get_dimens()
        }

        pub fn comp_set_size<T: Debug + Copy + Clone + PartialEq + Eq, O: Debug + Clone>(
            arg: &mut dyn Component<T, O>,
            dimens: T,
        ) {
            arg.set_dimens(dimens);
        }
    }

    //
    // fns that work with the super trait. (View)
    //

    pub mod supertrait_fns {
        use super::*;

        pub fn view_get_size<T: Debug + Copy + Clone + PartialEq + Eq>(arg: impl View<T>) -> T {
            arg.get_dimens()
        }

        pub fn view_set_size<T: Debug + Copy + Clone + PartialEq + Eq>(
            arg: &mut dyn View<T>,
            dimens: T,
        ) {
            arg.set_dimens(dimens);
        }

        // pub fn view_render<T: Debug + Copy + Clone + PartialEq + Eq, O: Debug + Clone>(
        //     _arg: impl View<T>,
        // ) -> O {
        //     panic!("Render method not implemented for View trait");
        // }
    }

    const SMALL: Dimens2D = Dimens2D {
        width: Width(100),
        height: Height(50),
        x: X(10),
        y: Y(20),
    };

    const LARGE: Dimens2D = Dimens2D {
        width: Width(200),
        height: Height(100),
        x: X(30),
        y: Y(40),
    };

    const OTHER: Dimens2D = Dimens2D {
        width: Width(300),
        height: Height(150),
        x: X(50),
        y: Y(60),
    };

    #[test]
    fn test_subtrait_fns() {
        let button_component = button::Button2D { dimens: SMALL };
        let label_component = label::Label2D { dimens: LARGE };

        let btn_output = subtrait_fns::comp_render(&button_component);
        let lbl_output = subtrait_fns::comp_render(&label_component);

        println!("{btn_output}");
        println!("{lbl_output}");

        assert_eq!(button_component.get_dimens(), SMALL);
        assert_eq!(label_component.get_dimens(), LARGE);
    }

    #[test]
    fn test_supertrait_fns() {
        let button_component = button::Button2D { dimens: SMALL };
        let label_component = label::Label2D { dimens: LARGE };

        let btn_size = supertrait_fns::view_get_size(button_component);
        let lbl_size = supertrait_fns::view_get_size(label_component);

        assert_eq!(btn_size, SMALL);
        assert_eq!(lbl_size, LARGE);
    }

    #[test]
    fn test_access_comp_as_view() {
        let mut button_component = button::Button2D { dimens: SMALL };
        let mut label_component = label::Label2D { dimens: SMALL };

        // View trait methods.
        button_component.set_dimens(LARGE);
        label_component.set_dimens(LARGE);

        assert_eq!(button_component.get_dimens(), LARGE);
        assert_eq!(label_component.get_dimens(), LARGE);

        // Component trait methods.
        let btn_output = button_component.render();
        let lbl_output = label_component.render();

        println!("{btn_output}");
        println!("{lbl_output}");
    }

    #[test]
    fn test_acc_comp_in_collection() {
        let mut components: Vec<&mut dyn Component<Dimens2D, String>> = vec![];

        // Before type erasure.
        let button_component = &mut button::Button2D { dimens: SMALL };
        let label_component = &mut label::Label2D { dimens: LARGE };

        assert_eq!(button_component.get_dimens(), SMALL);
        assert_eq!(label_component.get_dimens(), LARGE);

        components.push(button_component);
        components.push(label_component);

        // After type erasure.
        assert_eq!(components[0].get_dimens(), SMALL);
        assert_eq!(components[1].get_dimens(), LARGE);

        // Access components as Component.
        for component in components.iter() {
            let output = component.render();
            println!("{output}");
        }

        // Access components as View.
        for view in components.iter() {
            let dimens = view.get_dimens();
            println!(
                "Width: {}, Height: {}, X: {}, Y: {}",
                dimens.width.0, dimens.height.0, dimens.x.0, dimens.y.0
            );
        }

        // Set new dimens (on all the components, which are also views) in the collection.
        for view in components.iter_mut() {
            view.set_dimens(OTHER);
        }

        // Check if the dimens is set correctly.
        assert_eq!(components[0].get_dimens(), OTHER);
        assert_eq!(components[1].get_dimens(), OTHER);
    }
}

pub trait Shape3D {

}

pub struct ExtrudedShape<SHAPE: Shape3D> {
    shape: SHAPE
}

impl<SHAPE: Shape3D> ExtrudedShape<SHAPE> {

}
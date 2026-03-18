use bytemuck::{AnyBitPattern, NoUninit, Pod};
use vello::wgpu::{
    Buffer, BufferUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::backend::LumaBackend;

pub struct DynamicBuffer {
    pub(crate) inner: Buffer,
    len: usize,
}

pub fn size_of_vec<T>(vec: &Vec<T>) -> usize {
    vec.len() * std::mem::size_of::<T>()
}

impl DynamicBuffer {
    ///Gets the offset of the provided `T` on this buffer
    #[inline]
    fn offset_of<T>(&self) -> usize {
        self.len * std::mem::size_of::<T>()
    }

    ///Retrieves the size of this buffer
    /// s
    pub fn capacity(&self) -> usize {
        self.inner.size() as usize
    }
}
impl LumaBackend {
    pub fn create_dyn_buffer<T: Pod>(
        &self,
        data: &[T],
        permissions: BufferUsages,
    ) -> DynamicBuffer {
        let data = bytemuck::cast_slice(&data);
        DynamicBuffer {
            inner: self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("dynamic buffer creation"),
                contents: data,
                usage: permissions,
            }),
            len: data.len(),
        }
    }
    ///Inserts the provided `data` on the final of the provided `buffer` and returns its new capacity, if resized
    pub fn push_on_buffer<T: NoUninit + AnyBitPattern>(
        &self,
        buffer: &mut DynamicBuffer,
        data: &T,
    ) -> Option<usize> {
        let data = bytemuck::bytes_of(data);
        let offset = buffer.offset_of::<T>();

        if offset + data.len() > buffer.capacity() {
            let buf = buffer.inner.slice(..).get_mapped_range();
            let mut old_data: Vec<u8> =
                unsafe { std::slice::from_raw_parts(buf.as_ptr(), buf.len()) }.to_vec();
            old_data.resize(buffer.capacity() * 2, 0);
            old_data.extend_from_slice(data);

            buffer.inner = self
                .create_dyn_buffer(&old_data, buffer.inner.usage())
                .inner;
            Some(old_data.len())
        } else {
            self.queue.write_buffer(&buffer.inner, offset as u64, data);
            None
        }
    }
}

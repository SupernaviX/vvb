use anyhow::{anyhow, Result};
use jni::objects::{JFieldID, JObject};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{_jfieldID, jlong};
use jni::JNIEnv;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::{mem, ptr};

const LONG_TYPE: ReturnType = ReturnType::Primitive(Primitive::Long);

fn field_id_to_ptr(field: JFieldID) -> *mut _jfieldID {
    // safety: JFieldID is a repr(transparent) wrapper around this type
    unsafe { mem::transmute(field) }
}

fn ptr_to_field_id(ptr: *mut _jfieldID) -> JFieldID {
    // safety: JFieldID is a repr(transparent) wrapper around this type
    unsafe { mem::transmute(ptr) }
}

pub struct JavaBinding<T> {
    // JFieldId is a transparent wrapper around a raw pointer, store that
    field: AtomicPtr<_jfieldID>,
    // this doesn't actually own a `T`, but `field` is shared for all `T`
    // something like `PhantomData<TypeId<T>>` would be better
    _marker: PhantomData<AtomicPtr<T>>,
}

pub type JavaGetResult<'a, T> = Result<MutexGuard<'a, T>>;

impl<T> JavaBinding<T> {
    pub const fn new() -> Self {
        Self {
            field: Default::default(),
            _marker: Default::default(),
        }
    }
    pub fn init_value(&self, env: &mut JNIEnv, obj: JObject, value: T) -> Result<()> {
        let field = self.initialize_field_id(env, &obj)?;

        let mutex = Box::new(Mutex::new(value));
        let mutex_ptr = Box::into_raw(mutex) as jlong;
        env.set_field_unchecked(obj, field, mutex_ptr.into())?;

        Ok(())
    }

    pub fn get_value(&self, env: &mut JNIEnv, obj: JObject) -> JavaGetResult<T> {
        let mutex_ptr = self.get_mutex_ptr(env, &obj)?;
        // safe because we already null checked
        Ok(unsafe { (*mutex_ptr).lock().unwrap() })
    }

    pub fn drop_value(&self, env: &mut JNIEnv, obj: JObject) -> Result<()> {
        let mutex_ptr = self.get_mutex_ptr(env, &obj)?;
        let null_ptr = ptr::null_mut() as *mut Mutex<T>;
        env.set_field_unchecked(obj, self.get_field()?, (null_ptr as jlong).into())?;

        // safe because we already null checked
        let mutex = unsafe { Box::from_raw(mutex_ptr) };
        drop(mutex.into_inner().unwrap());
        Ok(())
    }

    fn initialize_field_id(&self, env: &mut JNIEnv, obj: &JObject) -> Result<JFieldID> {
        let class = env.auto_local(env.get_object_class(obj)?);
        let field = env.get_field_id(class, "_pointer", "J")?;
        self.field.store(field_id_to_ptr(field), Ordering::Relaxed);
        Ok(field)
    }

    fn get_field(&self) -> Result<JFieldID> {
        let field_ptr = self.field.load(Ordering::Relaxed);
        if field_ptr.is_null() {
            Err(anyhow!("Field was never initialized"))
        } else {
            Ok(ptr_to_field_id(field_ptr))
        }
    }

    // does null checks!
    fn get_mutex_ptr(&self, env: &mut JNIEnv, obj: &JObject) -> Result<*mut Mutex<T>> {
        let mutex_addr = env
            .get_field_unchecked(obj, self.get_field()?, LONG_TYPE)?
            .j()?;
        let mutex_ptr = mutex_addr as *mut Mutex<T>;
        if mutex_ptr.is_null() {
            Err(anyhow::anyhow!("Uninitialized Java value"))
        } else {
            Ok(mutex_ptr)
        }
    }
}

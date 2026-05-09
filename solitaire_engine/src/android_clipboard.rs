/// Android clipboard bridge via JNI.
///
/// Writes text to the system clipboard by calling into `ClipboardManager`
/// through the JNI. Only compiled and linked on `target_os = "android"`.
#[cfg(target_os = "android")]
pub fn set_text(text: &str) -> Result<(), String> {
    use bevy::android::ANDROID_APP;
    use jni::{
        objects::{JObject, JValue, JValueOwned},
        JavaVM,
    };

    let app = ANDROID_APP
        .get()
        .ok_or_else(|| "ANDROID_APP not initialized".to_string())?;

    // SAFETY: vm_as_ptr() returns the raw JavaVM* set up by the Android runtime.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }
        .map_err(|e| format!("JavaVM::from_raw: {e}"))?;

    let mut env = vm
        .attach_current_thread_permanently()
        .map_err(|e| format!("attach_current_thread: {e}"))?;

    // SAFETY: activity_as_ptr() is the NativeActivity jobject pointer —
    // valid for the lifetime of the process.
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as _) };

    (|| -> jni::errors::Result<()> {
        // ClipboardManager cm = activity.getSystemService("clipboard")
        let svc_name = JValueOwned::from(env.new_string("clipboard")?);
        let cm = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[svc_name.borrow()],
            )?
            .l()?;

        // ClipData clip = ClipData.newPlainText("link", text)
        let label = JValueOwned::from(env.new_string("link")?);
        let java_text = JValueOwned::from(env.new_string(text)?);
        let clip_class = env.find_class("android/content/ClipData")?;
        let clip = env
            .call_static_method(
                &clip_class,
                "newPlainText",
                "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
                &[label.borrow(), java_text.borrow()],
            )?
            .l()?;

        // cm.setPrimaryClip(clip)
        let clip_val = JValueOwned::Object(clip);
        env.call_method(
            &cm,
            "setPrimaryClip",
            "(Landroid/content/ClipData;)V",
            &[clip_val.borrow()],
        )?
        .v()
    })()
    .map_err(|e| format!("clipboard JNI: {e}"))
}

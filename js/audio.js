var ctx = null;
var buffer_size = 0;
var memory;

audio_set_mem = function (wasm_memory) {
    memory = wasm_memory;
}
audio_register_js_plugin = function (importObject) {
    importObject.env.audio_init = function (audio_buffer_size) {
        if (ctx != null) {
            console.error("Already inited");
        }
        window.startPause = 0;
        window.endPause = 0;
        document.addEventListener("visibilitychange", (e) => {
            // if (document.hidden) {
            //     window.startPause = performance.now() / 1000.0;
            // } else {
            //     window.endPause = performance.now() / 1000.0;
            // }
        }, false);

        ctx = new AudioContext();

        buffer_size = audio_buffer_size;

        if (ctx) {
            return true;
        } else {
            return false;
        }
    }
    importObject.env.audio_current_time = function () {
        return ctx.currentTime;
    }

    importObject.env.audio_samples = function (buffer_ptr, start_audio) {
        var buffer = ctx.createBuffer(2, buffer_size, ctx.sampleRate);
        var channel0 = buffer.getChannelData(0);
        var channel1 = buffer.getChannelData(1);
        var obuf = new Float32Array(memory.buffer, buffer_ptr, buffer_size * 2);
        for (var i = 0, j = 0; i < buffer_size * 2; i++) {

            channel0[i] = obuf[j++];
            channel1[i] = obuf[j++];
        }
        var bufferSource = ctx.createBufferSource();
        bufferSource.buffer = buffer;
        bufferSource.connect(ctx.destination);
        bufferSource.start(start_audio);
        var bufferSec = buffer_size / ctx.sampleRate;
        return bufferSec;
    }

    importObject.env.audio_sample_rate = function () {
        return ctx.sampleRate;
    }
}




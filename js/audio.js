var ctx = null;
var buffer_size = 0;

register_plugin = function (importObject) {
    importObject.env.audio_init = function (audio_buffer_size) {
        if (ctx != null) {
            return;
        }
        window.startPause = 0;
        window.endPause = 0;
        document.addEventListener("visibilitychange", (e) => {
            if (document.hidden) {
                window.startPause = performance.now() / 1000.0;
            } else {
                window.endPause = performance.now() / 1000.0;
            }
        }, false);

        // https://gist.github.com/kus/3f01d60569eeadefe3a1
        {
            audioContext = window.AudioContext || window.webkitAudioContext;
            ctx = new audioContext();
            var fixAudioContext = function (e) {
                console.log("fix");

                // Create empty buffer
                var buffer = ctx.createBuffer(1, 1, 22050);
                var source = ctx.createBufferSource();
                source.buffer = buffer;
                // Connect to output (speakers)
                source.connect(ctx.destination);
                // Play sound
                if (source.start) {
                    source.start(0);
                } else if (source.play) {
                    source.play(0);
                } else if (source.noteOn) {
                    source.noteOn(0);
                }

                // Remove events
                document.removeEventListener('touchstart', fixAudioContext);
                document.removeEventListener('touchend', fixAudioContext);
                document.removeEventListener('mousedown', fixAudioContext);
            };
            // iOS 6-8
            document.addEventListener('touchstart', fixAudioContext);
            // iOS 9
            document.addEventListener('touchend', fixAudioContext);

            document.addEventListener('mousedown', fixAudioContext);
        }

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
        var obuf = new Float32Array(wasm_memory.buffer, buffer_ptr, buffer_size * 2);
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

    importObject.env.audio_pause_state = function () {
        var duration = window.endPause - window.startPause;
        if (duration > 0) {
            window.endPause = 0;
            window.startPause = 0;
            return duration;
        } else if (window.startPause > 0) {
            return -1;
        } else {
            return 0.0;
        }
    }
}

miniquad_add_plugin({ register_plugin });
package flash.system {
    import flash.events.EventDispatcher;
    public final class Worker extends EventDispatcher {
        public static const current:Worker = new Worker();
        public static const isSupported:Boolean = false;

        public const isPrimordial:Boolean = true;
        public const state:WorkerState = null;

        public function Worker() {
            if (!current) return;
            throw new ArgumentError("Error #2012: Worker$ class cannot be instantiated.", 2012)
        }

        public function createMessageChannel(receiver:Worker):MessageChannel {
            return null;
        }

        public function getSharedProperty(key:String):* {
            return undefined;
        }

        public function setSharedProperty(key:String, value:*):void {}

        public function start():void {}
        public function terminate():Boolean {
            return false;
        }
    }
}
package flash.system {
    import flash.events.EventDispatcher;
    public final class MessageChannel extends EventDispatcher {
        public function MessageChannel() {
            throw new ArgumentError("Error #2012: MessageChannel$ class cannot be instantiated.", 2012)
        }
    }
}
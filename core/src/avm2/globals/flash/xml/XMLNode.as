package flash.xml {
    [Ruffle(InstanceAllocator)]
    public class XMLNode {
        public function XMLNode(type:uint, value:String) {
            this.init(type, value);
        }

        private native function init(type:uint, value:String): void;
    }
}
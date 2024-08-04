import {
  useDialog,
  useLoadingBar,
  useMessage,
  useNotification,
} from "naive-ui";
import { defineComponent, useSlots } from "vue";

const GlobalInject = defineComponent({
  name: "GlobalInject",
  setup() {
    const slots = useSlots();
    let that = window as any;
    // mount
    that.$message = useMessage();
    that.$dialog = useDialog();
    that.$notification = useNotification();
    that.$loadingBar = useLoadingBar();

    return () => <>{slots.default!()}</>;
  },
});

export default GlobalInject;

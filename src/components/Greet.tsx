import { dialog, invoke } from "@tauri-apps/api";
import { NButton, NForm, NFormItem } from "naive-ui";
import { defineComponent, ref } from "vue";
const Greet = defineComponent({
  name: "Greet",
  emits: [],
  setup() {
    const greetMsg = ref("");

    return () => {
      return (
        <NForm>
          <NFormItem>
            <input
              onDragover={(e) => {
                e.preventDefault();
              }}
              onDrop={(e) => {
                console.log("Log-- ", e, "e");
              }}
              onChange={async (e) => {
                console.log("Log-- ", e, "e");
              }}
              type="file"
            />
            <NButton
              onClick={async () => {
                const file = await dialog.open({
                  directory: false,
                  multiple: false,
                });
                greetMsg.value = await invoke("towebp", { name: file });
              }}
            >
              towebp
            </NButton>
            <NButton
              onClick={async () => {
                const file = await dialog.open({
                  directory: false,
                  multiple: false,
                });
                console.log("Log-- ", file, "file");
                greetMsg.value = await invoke("testffmpeg", { name: file });
              }}
            >
              testffmpeg
            </NButton>
            <NButton
              onClick={async () => {
                greetMsg.value = ((await invoke("get_assets")) as any).join(
                  "\\"
                );

                console.log("Log-- ", greetMsg.value, " greetMsg.value");
              }}
            >
              get_assets
            </NButton>
            <NButton
              onClick={async () => {
                greetMsg.value = (
                  (await invoke("get_last_assets")) as any
                ).join("\\");

                console.log("Log-- ", greetMsg.value, " greetMsg.value");
              }}
            >
              get_last_assets
            </NButton>
            <NButton
              onClick={async () => {
                greetMsg.value = ((await invoke("test_libvips")) as any).join(
                  "\\"
                );

                console.log("Log-- ", greetMsg.value, " greetMsg.value");
              }}
            >
              test_libvips
            </NButton>
          </NFormItem>
          <NFormItem>
            <p>{greetMsg.value}</p>
          </NFormItem>
        </NForm>
      );
    };
  },
});
export default Greet;

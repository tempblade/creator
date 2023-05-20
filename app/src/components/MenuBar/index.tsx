import { FC } from "react";
import * as Menubar from "@radix-ui/react-menubar";
import { ChevronRightIcon } from "@radix-ui/react-icons";
import { open, save } from "@tauri-apps/api/dialog";

const MenuBarTrigger: FC<{ label: string }> = ({ label }) => {
  return (
    <Menubar.Trigger className="py-2 dark:text-gray-300 px-3 transition-colors hover:bg-indigo-700  outline-none select-none font-medium leading-none rounded text-[13px] flex items-center justify-between gap-[2px]">
      {label}
    </Menubar.Trigger>
  );
};

const MenuBarSubTrigger: FC<{ label: string }> = ({ label }) => {
  return (
    <Menubar.SubTrigger
      className="group dark:text-gray-300 text-[13px] hover:bg-indigo-800 transition-colors leading-none
       text-indigo11 rounded flex items-center h-[25px] px-[10px] relative select-none outline-none 
       data-[state=open]:bg-indigo data-[state=open]:text-white data-[highlighted]:bg-gradient-to-br 
       data-[highlighted]:from-indigo9 data-[highlighted]:to-indigo10 data-[highlighted]:text-indigo1 
       data-[highlighted]:data-[state=open]:text-indigo1 data-[disabled]:text-mauve8 data-[disabled]:pointer-events-none"
    >
      {label}
      <div className="ml-auto pl-5 text-mauve9 group-data-[highlighted]:text-white group-data-[disabled]:text-mauve8">
        <ChevronRightIcon />
      </div>
    </Menubar.SubTrigger>
  );
};

const MenuBarItem: FC<{ label: string; onClick?: () => void }> = ({
  label,
  onClick,
}) => {
  return (
    <Menubar.Item
      onClick={onClick}
      className="group dark:text-white text-[13px] leading-none 
      rounded flex items-center h-[25px] px-[10px] 
      relative select-none outline-none hover:bg-indigo-800
      data-[disabled]:pointer-events-none transition-colors"
    >
      {label}
    </Menubar.Item>
  );
};

const MenuBarSeperator = () => {
  return <Menubar.Separator className="h-[1px] bg-slate-500 m-[5px]" />;
};

const MenuBar = () => {
  const menuBarContentClassName =
    "min-w-[220px] bg-gray-800 rounded-md p-[5px]";

  const menuBarSubContentClassName =
    "min-w-[220px] bg-gray-800 rounded-md p-[5px]";

  return (
    <Menubar.Root className="flex bg-gray-900 p-[3px] ">
      <Menubar.Menu>
        <MenuBarTrigger label="File" />
        <Menubar.Portal>
          <Menubar.Content
            className={menuBarContentClassName}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <MenuBarItem label="New File" />
            <MenuBarItem
              onClick={() => open({ multiple: false })}
              label="Open File"
            />
            <MenuBarItem
              onClick={() =>
                save({
                  title: "Save Project",
                  defaultPath: "project.tbcp",
                }).then((val) => {
                  console.log(val);
                })
              }
              label="Save"
            />
            <MenuBarItem onClick={() => save()} label="Save as" />
            <MenuBarSeperator />
            <Menubar.Sub>
              <MenuBarSubTrigger label="Export as ..." />

              <Menubar.Portal>
                <Menubar.SubContent
                  className={menuBarSubContentClassName}
                  alignOffset={-5}
                >
                  <MenuBarItem label=".mp4" />
                  <MenuBarItem label=".gif" />
                  <MenuBarItem label=".mov" />
                  <MenuBarItem label=".webm" />
                  <MenuBarItem label=".webp" />
                </Menubar.SubContent>
              </Menubar.Portal>
            </Menubar.Sub>
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>

      <Menubar.Menu>
        <MenuBarTrigger label="Edit" />

        <Menubar.Portal>
          <Menubar.Content
            className={menuBarContentClassName}
            align="start"
            sideOffset={5}
            alignOffset={-3}
          >
            <MenuBarItem label="Undo" />
            <MenuBarItem label="Redo" />
            <MenuBarItem label="Copy" />
            <MenuBarItem label="Paste" />
          </Menubar.Content>
        </Menubar.Portal>
      </Menubar.Menu>
    </Menubar.Root>
  );
};

export default MenuBar;

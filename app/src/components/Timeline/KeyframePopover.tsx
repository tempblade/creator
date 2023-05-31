import { PopoverContent, PopoverPortal } from "@radix-ui/react-popover";
import { Popover } from "components/Popover";
import { Keyframe } from "primitives/Keyframe";
import { FC } from "react";
import { z } from "zod";

const KeyframePopover: FC<{
  open: boolean;
  keyframe: z.input<typeof Keyframe>;
  onUpdate: (k: z.input<typeof Keyframe>) => void;
}> = ({ open, keyframe, onUpdate }) => {
  return (
    <div>
      <Popover open={open}>
        <PopoverContent>
          <label>
            <span className="label">Value</span>
            <input
              onChange={(e) =>
                onUpdate({ ...keyframe, value: Number(e.target.value) })
              }
              value={keyframe.value}
            />
          </label>
        </PopoverContent>
      </Popover>
    </div>
  );
};

export default KeyframePopover;

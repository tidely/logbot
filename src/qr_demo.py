#!/usr/bin/env python3

from PIL import Image
from io import BytesIO
from pyzbar.pyzbar import decode
from picamera2 import Picamera2, Preview


def main():
    camera = Picamera2()

    cam_config = camera.create_still_configuration(
        main={"size": (1920, 1080)},
        lores={"size": (640, 480)},
        display="lores"
    )

    camera.configure(cam_config)

    camera.start_preview(Preview.NULL)
    camera.start()

    image_data = BytesIO()
    camera.capture_file(image_data, format="png")

    image_obj = Image.open(image_data)
    decoded_data = decode(image_obj)

    if len(decoded_data) != 0:
        print(decoded_data[0].data.decode())


if __name__ == '__main__':
    main()

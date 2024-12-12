#!/usr/bin/env python3

import requests

from PIL import Image
from PIL import ImageFile
from io import BytesIO

from pyzbar.pyzbar import decode

from flask import Blueprint, render_template
from flask_login import login_required

ImageFile.LOAD_TRUNCATED_IMAGES = True

controlpanel_blueprint = Blueprint("controlpanel", __name__)


@controlpanel_blueprint.route('/', methods=["GET"])
def index():
    return render_template("index.html")


@controlpanel_blueprint.route('/controlpanel', methods=["GET", "POST"])
@login_required
def controlpanel():
    # This page contains all of the information we want to show as well as the mode selection.
    return render_template("controlpanel.html")


@controlpanel_blueprint.route('/qrcode', methods=["POST"])
def qrcode():
    resp = requests.get("http://127.0.0.1/stream", stream=True)
    stream_content_generator = resp.iter_content(chunk_size=64)

    headers = next(stream_content_generator).split(b"\n", 4)
    content_length = int(headers[2].split(b" ")[1])

    beginning_of_image_data = headers[4]

    content_length = content_length - len(beginning_of_image_data)

    image_data = beginning_of_image_data
    new_image_generator = resp.iter_content(chunk_size=8192)
    while len(image_data) < content_length:
        image_data += next(new_image_generator)

    image_obj = Image.open(BytesIO(image_data[:content_length]))

    decoded_data = decode(image_obj)

    if len(decoded_data) != 0:
        return decoded_data[0].data.decode()
    else:
        return "Reading QR code failed!"

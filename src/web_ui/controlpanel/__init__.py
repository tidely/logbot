#!/usr/bin/env python3

import os
from flask import Flask
from flask_login import LoginManager
from flask_sqlalchemy import SQLAlchemy
from werkzeug.middleware.proxy_fix import ProxyFix

from dotenv import load_dotenv
load_dotenv()

db = SQLAlchemy()


def create_app():
    app = Flask(__name__)
    app.config["SECRET_KEY"] = os.getenv("SECRET_KEY")
    assert app.config["SECRET_KEY"] is not None, "Flask SECRET_KEY seems to be nonexistent."
    app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///db.sqlite"
    
    app.wsgi_app = ProxyFix(
        app.wsgi_app, x_for=1, x_proto=1, x_host=1, x_prefix=1
    )

    db.init_app(app)

    login_manager = LoginManager()
    login_manager.login_view = "auth.login"
    login_manager.init_app(app)

    from .models import User

    @login_manager.user_loader
    def load_user(user_id):
        return User.query.get(int(user_id))

    from .auth import auth_blueprint
    app.register_blueprint(auth_blueprint)

    from .controlpanel import controlpanel_blueprint
    app.register_blueprint(controlpanel_blueprint)

    with app.app_context():
        db.create_all()

    return app
